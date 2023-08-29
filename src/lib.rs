
use std::io::{self, Read, Write, BufRead};
use std::os::fd::{AsRawFd, RawFd};
use std::mem::MaybeUninit;

// awful trait aliases

/// # A trait for shortening type names
/// Anything that implements both `Read` and `AsRawFd` automatically implement
/// `TermIn`
pub trait TermIn:  Read  + AsRawFd {}
/// # A trait for shortening type names
/// Anything that implements both `Write` and `AsRawFd` automatically implement
/// `TermOut` 
pub trait TermOut: Write + AsRawFd {}

impl<T> TermIn  for T where T: Read  + AsRawFd {}
impl<T> TermOut for T where T: Write + AsRawFd {}
//

const TCSANOW: i32 =        0;

const ICANON : u32 = 0o000002;
const ECHO   : u32 = 0o000010;

#[inline(always)]
fn io_result(result: i32) -> io::Result<()> {
    if result == 0 { Ok(()) }
    else { Err(io::Error::last_os_error()) }
}

const NCCS: usize = 32;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
struct termios {
    c_iflag: u32,
    c_oflag: u32,
    c_cflag: u32,
    c_lflag: u32,
    c_line: u8,
    c_cc: [u8; NCCS],
    c_ispeed: u32,
    c_ospeed: u32,
}
#[link(name = "c")]
extern "C" {
    fn tcsetattr(fd: RawFd, optional_actions: i32, termios_p: *const termios) -> i32;
    fn tcgetattr(fd: RawFd, termios_p: *mut termios) -> i32;
}

impl termios {
    fn copy_from(&mut self, other: &Self) {    
        self.c_iflag = other.c_iflag;
        self.c_oflag = other.c_oflag;
        self.c_cflag = other.c_cflag;
        self.c_lflag = other.c_lflag;
        self.c_line = other.c_line;
        self.c_cc = other.c_cc;
        self.c_ispeed = other.c_ispeed;
        self.c_ospeed = other.c_ospeed;
    }
}

/// # Terminal
/// Stores the current termios stuff and derives a new termios object. It also
/// reverts changes on drop. All fields should be treated as private, mutating
/// any of them (pointer magic included) will break the system.
///
/// If `I` implements `BufRead`, then `Terminal` will implement `BufRead`.
pub struct Terminal<I: TermIn, O: TermOut> {
    stdin: I,
    stdout: O,
    prior: termios,
    current: termios,
}

impl<I: TermIn, O: TermOut> Terminal<I, O> {
    /// constructs a new `Terminal` object and puts it in control.
    /// If you're using stdin/stdout, it's recommended to lock them first
    /// since it gives you `StdinLock`/`StdoutLock`, which gives you the
    /// `BufRead` trait.
    pub fn new(input: I, output: O) -> io::Result<Self> {
        let fd = input.as_raw_fd();
        let mut temp = MaybeUninit::uninit();
        let temp = unsafe {
            io_result(tcgetattr(fd, temp.assume_init_mut()))?;
            temp.assume_init()
        };
        Ok(Terminal{stdin: input, stdout: output, current: temp.clone(), prior: temp})
    }
    /// prep a transform object for changing the current termios object.
    pub fn change<'a>(&'a mut self) -> Transform<'a, I, O> {
        Transform{config: self.current.clone(), source: self}
    }

    /// # foreground
    /// sets the foreground colour
    pub fn foreground(&mut self, col: Colour) -> io::Result<()> {
        match col {
            Colour::Term(x) => self.write(format!("\x1b[{}m", 30 + (x & 0x07)
                                                 + if x & 0x08 == 0x08 {60}
                                                   else {0})
                                          .as_bytes( )),
            Colour::Byte(x) => self.write(format!("\x1b[38;5;{x}m")
                                          .as_bytes()),
            Colour::RGB(r, g, b) => self.write(format!("\x1b[38;2;{r};{g};{b}m")
                                               .as_bytes()),
        }.map(|_| ())
    }
    /// # background
    /// sets the background colour
    pub fn background(&mut self, col: Colour) -> io::Result<()> {
        match col {
            Colour::Term(x) => self.write(format!("\x1b[{}m", 40 + (x & 0x07)
                                                 + if x & 0x08 == 0x08 {60}
                                                   else {0})
                                          .as_bytes( )),
            Colour::Byte(x) => self.write(format!("\x1b[48;5;{x}m")
                                          .as_bytes()),
            Colour::RGB(r, g, b) => self.write(format!("\x1b[48;2;{r};{g};{b}m")
                                               .as_bytes()),
        }.map(|_| ())
    }
    pub fn style_clear(&mut self) -> io::Result<()> {
        self.write(b"\x1b[0m").map(|_| ())
    }

    pub fn pull_utf8(&mut self) -> io::Result<Option<char>> {
        // this is so fun :)
        // hoping that the standard is true with a max of 4
        let mut buff = [0u8; 4];
        self.read_exact(&mut buff[0 .. 1])?;
        let elen = buff[0].leading_ones() as usize;
        if elen == 0 { return Ok(Some(buff[0] as char)) }
        else if elen == 1 { return Ok(None) }
        assert!(elen <= 4, "The Standard Lied!!");
        self.read_exact(&mut buff[1 .. elen])?;
        
        let mut loc : u32 = (buff[0] & (0xff >> elen)) as u32;
        for x in &buff[1 .. elen] {
            loc = (loc << 6) + (x & 0x3fu8) as u32;
        }
        Ok(char::from_u32(loc))
    }
}

impl<I: TermIn, O: TermOut> Write for Terminal<I, O> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}
impl<I: TermIn, O: TermOut> Read for Terminal<I, O> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdin.read(buf)
    }
}

impl<I: TermIn + BufRead, O: TermOut> BufRead for Terminal<I, O> {
    fn consume(&mut self, amt: usize) {
        self.stdin.consume(amt)
    }
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.stdin.fill_buf()
    }
}

impl<I: TermIn, O: TermOut> Drop for Terminal<I, O> {
    /// automatically revert termios object.
    fn drop(&mut self) {
        unsafe { tcsetattr(self.stdout.as_raw_fd(), TCSANOW, &self.prior) };
    }
}


// Apologies in advance for this horrible code
/// # Transform
/// A simple way for editing a termios object. Changes are stored until
/// commited. This is to avoid weird transition states like what you see in
/// some Nim libraries ( I've not used any in other languages, ncurses is
/// really solid though).
///
/// A transfrom should not be constructed on it's own, and should only be made
/// via the `change` method on `Terminal`.
pub struct Transform<'a, I: TermIn, O: TermOut> {
    source: &'a mut Terminal<I, O>,
    config: termios, 
}

impl<'a, I: TermIn, O: TermOut> Transform<'a, I, O> {
    /// commits changes and 
    pub fn commit(self) -> io::Result<()> {
        unsafe {
            io_result(tcsetattr(self.source.stdout.as_raw_fd(), TCSANOW, &self.config))?;
        };
        self.source.current.copy_from(&self.config);
        Ok(())
    }
    pub fn canonical(&mut self, value: bool) -> &mut Self {
       self.config.c_lflag &= !ICANON;
       if value { self.config.c_lflag |= ICANON; }
       self
    }
    pub fn echo(&mut self, value: bool) -> &mut Self {
       self.config.c_lflag &= !ECHO;
       if value { self.config.c_lflag |= ECHO; }
       self
    }
}

pub trait TermTransform: Sized {
    fn commit
        <I: TermIn, O: TermOut>
        (self, t: &mut Terminal<I, O>) -> io::Result<()>;
    fn fullscreen(self, x: bool) -> Fullscreen<Self> {
        Fullscreen(self, x)
    }
    fn foreground(self, c: Colour) -> Foreground<Self> {
        Foreground(self, c)
    }
    fn background(self, c: Colour) -> Background<Self> {
        Background(self, c)
    }
    fn echo(self, x: bool) -> Echo<Self> {
        Echo(self, x)
    }
    fn canon(self, x: bool) -> Canon<Self> {
        Canon(self, x)
    }
}


pub struct Fullscreen<T: TermTransform>(T, bool);
pub struct Foreground<T: TermTransform>(T, Colour);
pub struct Background<T: TermTransform>(T, Colour);
pub struct Echo<T: TermTransform>(T, bool);
pub struct Canon<T: TermTransform>(T, bool);

impl<T: TermTransform> TermTransform for Fullscreen<T> {
    fn commit<I: TermIn, O: TermOut>(self, t: &mut Terminal<I ,O>)
        -> io::Result<()>
    {
        self.0.commit(t)?;
        if self.1 {
            t.write(b"\x1b[?1049h")?;
        } else {
            t.write(b"\x1b[?1049l")?;
        }
        Ok(())
    }
    /*
    fn fullscreen(self, x) -> Fullscreen<T> {
        Fullscreen(self.0, x)
    }
    */
}

impl TermTransform for () {
    fn commit<I: TermIn, O: TermOut>(self, _t: &mut Terminal<I ,O>)
        -> io::Result<()>
    {
        Ok(())
    }
}

impl<T: TermTransform> TermTransform for Foreground<T> {
    fn commit<I: TermIn, O: TermOut>(self, t: &mut Terminal<I ,O>)
        -> io::Result<()>
    {
        self.0.commit(t)?;
        t.foreground(self.1)
    }
}
impl<T: TermTransform> TermTransform for Background<T> {
    fn commit<I: TermIn, O: TermOut>(self, t: &mut Terminal<I ,O>)
        -> io::Result<()>
    {
        self.0.commit(t)?;
        t.background(self.1)
    }
}
impl<T: TermTransform> TermTransform for Echo<T> {
    fn commit<I: TermIn, O: TermOut>(self, t: &mut Terminal<I, O>)
        -> io::Result<()>
    {
       t.current.c_lflag &= !ECHO;
       if self.1 { t.current.c_lflag |= ECHO; }
       unsafe {
           io_result(tcsetattr(t.stdout.as_raw_fd(), TCSANOW, &t.current))
       }
    }
}
impl<T: TermTransform> TermTransform for Canon<T> {
    fn commit<I: TermIn, O: TermOut>(self, t: &mut Terminal<I, O>)
        -> io::Result<()>
    {
       t.current.c_lflag &= !ICANON;
       if self.1 { t.current.c_lflag |= ICANON; }
       unsafe {
           io_result(tcsetattr(t.stdout.as_raw_fd(), TCSANOW, &t.current))
       }
    }
}
/// # Colour
/// a simple enum type for representing the different colour formats that are
/// supported via ANSI escape codes.
/// - Term: for the base colour codes (0 to 15 including BRIGHT flag)
///   [source](https://en.wikipedia.org/wiki/ANSI_escape_code#3-bit_and_4-bit)
///
/// - Byte: for 8-bit colour codes
///   [source](https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit)
///
/// - RGB:  exactly what it says it is
///   [source](https://en.wikipedia.org/wiki/ANSI_escape_code#24-bit)
///
// TODO: implement colour mixing
pub enum Colour {
    Term(u8),
    Byte(u8),
    RGB(u8,u8,u8),
}


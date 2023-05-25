
use std::io::{self, Read, Write, BufRead};
use std::os::fd::{AsRawFd, RawFd};
use std::mem::MaybeUninit;

const TCSANOW: i32 = 0;

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

pub struct Terminal<I: Read + AsRawFd, O: Write + AsRawFd> {
    stdin: I,
    stdout: O,
    prior: termios,
}

impl<I: Read + AsRawFd, O: Write + AsRawFd> Terminal<I, O> {
    pub fn new(input: I, output: O) -> io::Result<Self> {
        let fd = input.as_raw_fd();
        let mut temp = MaybeUninit::uninit();
        let temp = unsafe {
            io_result(tcgetattr(fd, temp.assume_init_mut()))?;
            temp.assume_init()
        };
        Ok(Terminal{stdin: input, stdout: output, prior: temp})
    }
}

impl<I: Read + AsRawFd, O: Write + AsRawFd> Write for Terminal<I, O> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}
impl<I: Read + AsRawFd, O: Write + AsRawFd> Read for Terminal<I, O> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdin.read(buf)
    }
}

impl<I: Read + AsRawFd + BufRead, O: Write + AsRawFd> BufRead for Terminal<I, O> {
    fn consume(&mut self, amt: usize) {
        self.stdin.consume(amt)
    }
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.stdin.fill_buf()
    }
}

impl<I: Read + AsRawFd, O: Write + AsRawFd> Drop for Terminal<I, O> {
    fn drop(&mut self) {
        unsafe { tcsetattr(self.stdout.as_raw_fd(), TCSANOW, &self.prior) };
    }
}




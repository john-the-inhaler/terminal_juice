
extern crate terminal_juice as TJ;
use TJ::*;

use std::io::{self, stdin, stdout, Write, BufRead};

pub fn main() -> io::Result<()> {
    let mut terminal = TJ::Terminal::new(stdin().lock(), stdout().lock())?;
    
    terminal.write(b"type something: ")?;
    terminal.flush()?;

    ().echo(false).commit(&mut terminal)?;
    let mut buff = String::new();
    terminal.read_line(&mut buff)?;
    ().echo(true).commit(&mut terminal)?;

    for cur in unsafe{buff.as_mut_str().as_bytes_mut()} {
        if cur.clone() & 0x80  == 0  && (1 ..= 26).contains(&(cur.clone() & 0x1f)) {
            *cur ^= 0x20;
        }
    }

    terminal.write(b"\n")?;
    terminal.write(buff.as_bytes())?;
    terminal.flush()?;
    Ok(())
) }

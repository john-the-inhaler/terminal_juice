
extern crate terminal_juice as TJ;

use std::io::{self, stdin, stdout, Write};


fn main() -> io::Result<()> {
    let mut terminal = TJ::Terminal::new(stdin().lock(), stdout().lock())?;
    for i in 0 .. 16 {
        let ri = 15 - i;
        terminal.foreground(TJ::Colour::Term(i as u8))?;
        terminal.background(TJ::Colour::Term(ri as u8))?;
        terminal.write(format!("The colour of term col {i:>02}:{ri:>02}")
                               .as_bytes())?;
        terminal.style_clear()?;
        terminal.write(b"\n")?;
        terminal.flush()?;
    }
    Ok(())
}

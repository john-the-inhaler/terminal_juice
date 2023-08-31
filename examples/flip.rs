extern crate terminal_juice as TJ;
use std::io::{self, Read, Write, BufRead};

use crate::TJ::TermTransform;

const GRID_HEIGHT : usize = 10;
const GRID_WIDTH  : usize = 10;


type Grid = [[bool; GRID_WIDTH]; GRID_HEIGHT];

fn flip_at(grid: &mut Grid, col: usize, row: usize) {
    grid.get_mut(row).and_then(|i| i.get_mut(col).map(|x| *x = !x.clone()));
}

fn flip(grid: &mut Grid, col: usize, row: usize) {
    flip_at(grid, col, row);
    if col > 0 { flip_at(grid, col - 1, row) }
    if row > 0 { flip_at(grid, col, row - 1) }
    if col + 1 < GRID_WIDTH  { flip_at(grid, col + 1, row) }
    if row + 1 < GRID_HEIGHT { flip_at(grid, col, row + 1) }
}

fn render_scene
    (terminal: &mut TJ::Terminal<io::StdinLock, io::StdoutLock>, 
     grid: &Grid, x: usize, y: usize)
    -> io::Result<()>
{
    // assuming we've homed the cursor
    for row in 0 .. GRID_HEIGHT {
        for col in 0 .. GRID_WIDTH {
            if row == y && col == x {
                terminal.style_direct(7)?;
                terminal.write(if grid[row][col] {b"#"} else {b"."})?;
                terminal.style_direct(0)?;
            } else {
                terminal.write(if grid[row][col] {b"#"} else {b"."})?;
            }
        }
        terminal.write(b"\n")?;
    }
    terminal.flush()?;
    Ok(())
}

fn detach<'a, 'b: 'a, T: ?Sized>(x: &'a T) -> &'b T {
    unsafe { *std::ptr::addr_of!(x).cast() }
}

pub fn main() -> io::Result<()> {
    let mut terminal = TJ::Terminal::new(io::stdin().lock(), io::stdout().lock())?;
    ().echo(false).canon(false).commit(&mut terminal)?;
    terminal.set_vmin(1)?;
    // allocating screen space
    {
        write!(&mut terminal, "\x1b[{GRID_HEIGHT}S\x1b[{GRID_HEIGHT}A")?;
    }


    let row: usize;
    {
        terminal.write(b"\x1b[6n")?;
        terminal.flush()?;
        let mut buff = [0u8; 16];
        let bufflen  = terminal.read(&mut buff)?;
        // eprintln!("bufflen = {bufflen}");
        assert!(bufflen >= 6);
        assert_eq!(buff[0], 0x1b);

        let view = unsafe {std::str::from_utf8_unchecked(&buff[2.. bufflen])};
        let mut end_first = 0;
        while let b'0' ..= b'9' = view.as_bytes()[end_first] { end_first += 1; }
        
        // eprintln!("parsing {:?}", &view[0 .. end_first]);
        row = view[0 .. end_first].parse().expect("cannot find cursor");
        // eprintln!("The cursor was on row {row}");
    }
    terminal.set_vmin(1)?;
    terminal.flush()?;
    
    let (mut x, mut y) = (0, 0);

    let mut grid = [[false; GRID_WIDTH]; GRID_HEIGHT];
    'outer: loop {
        write!(&mut terminal, "\x1b[{row};1H")?;
        render_scene(&mut terminal, &grid, x, y)?;
        match terminal.pull_utf8()?.unwrap_or('\0') {
            'q' => break,
            '\x1b' => {
                let buff = detach(terminal.fill_buf()?);
                let mut end_part = 0;
                while let Some(x) = buff.get(end_part).copied() {
                    end_part += 1;
                    if (x as char).is_ascii_alphabetic() { break }
                }
                terminal.consume(end_part.saturating_sub(1));
                match buff[end_part - 1] {
                    b'A' => y = y.saturating_sub(1),
                    b'B' => y = y.saturating_add(1).min(GRID_HEIGHT - 1),
                    b'D' => x = x.saturating_sub(1),
                    b'C' => x = x.saturating_add(1).min(GRID_WIDTH - 1),
                    _ => ()
                }
            }
            ' ' => 'inner: {
                flip(&mut grid, x, y);
                for row in grid {
                    for cell in row {if !cell {break 'inner}}
                }
                terminal.write(b"YOU WON!\n")?;
                write!(&mut terminal, "\x1b[{row};1H")?;
                render_scene(&mut terminal, &grid, x, y)?;
                write!(&mut terminal, "\x1b[{};1H", GRID_HEIGHT + 1)?;
                return Ok(());
            }
            _ => ()
        }
    }

    Ok(())
}

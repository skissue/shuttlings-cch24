use std::fmt::Display;

#[derive(Clone, Copy)]
enum Tile {
    Empty,
    Cookie,
    Milk,
}

impl Tile {
    fn emoji(&self) -> &str {
        match self {
            Tile::Empty => "⬛",
            Tile::Cookie => "🍪",
            Tile::Milk => "🥛",
        }
    }
}

pub struct Connect4 {
    board: [[Tile; 4]; 4],
}

impl Connect4 {
    pub fn empty() -> Connect4 {
        Connect4 {
            board: [[Tile::Empty; 4]; 4],
        }
    }
}

impl Display for Connect4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.board.len() {
            f.write_str("⬜")?;
            for x in 0..self.board[0].len() {
                f.write_str(self.board[x][y].emoji())?;
            }
            f.write_str("⬜\n")?;
        }
        f.write_str("⬜⬜⬜⬜⬜⬜\n")?;
        Ok(())
    }
}

use std::fmt::Display;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
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

pub enum MoveError {
    InvalidColumn,
    ColumnFull,
    GameOver,
}

#[derive(PartialEq, Eq)]
enum GameStatus {
    Ongoing,
    NoWinner,
    Winner(Tile),
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

    pub fn play(&mut self, team: Tile, col_idx: usize) -> Result<(), MoveError> {
        if col_idx < 1 || col_idx > 4 {
            return Err(MoveError::InvalidColumn);
        }
        if self.winner() != GameStatus::Ongoing {
            return Err(MoveError::GameOver);
        }

        let column = &mut self.board[col_idx - 1];
        let Some(drop_idx) = column.iter().rev().position(|&t| t == Tile::Empty) else {
            return Err(MoveError::ColumnFull);
        };
        column[3 - drop_idx] = team;

        Ok(())
    }

    fn winner(&self) -> GameStatus {
        // Rows
        for y in 0..4 {
            let initial = self.board[0][y];
            if initial != Tile::Empty
                && initial == self.board[1][y]
                && initial == self.board[2][y]
                && initial == self.board[3][y]
            {
                return GameStatus::Winner(initial);
            }
        }

        // Columns
        for x in 0..4 {
            let initial = self.board[x][0];
            if initial != Tile::Empty
                && initial == self.board[x][1]
                && initial == self.board[x][2]
                && initial == self.board[x][3]
            {
                return GameStatus::Winner(initial);
            }
        }

        // Diagonals
        let initial = self.board[0][0];
        if initial != Tile::Empty
            && initial == self.board[1][1]
            && initial == self.board[2][2]
            && initial == self.board[3][3]
        {
            return GameStatus::Winner(initial);
        }
        let initial = self.board[3][0];
        if initial != Tile::Empty
            && initial == self.board[2][1]
            && initial == self.board[1][2]
            && initial == self.board[0][3]
        {
            return GameStatus::Winner(initial);
        }

        // All filled
        if self
            .board
            .iter()
            .flat_map(|r| r.iter())
            .all(|t| *t != Tile::Empty)
        {
            return GameStatus::NoWinner;
        }

        GameStatus::Ongoing
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

        match self.winner() {
            GameStatus::Winner(tile) => write!(f, "{} wins!\n", tile.emoji())?,
            GameStatus::NoWinner => write!(f, "No winner.\n")?,
            _ => {}
        }

        Ok(())
    }
}

use anyhow::{bail, Result};
use rand::rngs::StdRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Deref;

pub const BOARD_SIZE: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connect4 {
    board: [[Cell; BOARD_SIZE]; BOARD_SIZE],
}

impl Connect4 {
    pub fn new() -> Self {
        Self {
            board: [[Cell::default(); BOARD_SIZE]; BOARD_SIZE],
        }
    }

    pub fn random(rng: &mut StdRng) -> Self {
        let mut connect4 = Self::new();
        for row in connect4.board.iter_mut() {
            for cell in row.iter_mut() {
                let player = if rng.gen::<bool>() {
                    Player::Cookie
                } else {
                    Player::Milk
                };
                *cell = player.into();
            }
        }
        connect4
    }

    pub fn play(&mut self, player: Player, column: usize) -> Result<()> {
        if column >= BOARD_SIZE {
            bail!("Invalid column");
        }
        for row in self.board.iter_mut().rev() {
            if row[column].is_none() {
                row[column] = player.into();
                return Ok(());
            }
        }
        bail!("Column full");
    }

    pub fn board_full(&self) -> bool {
        self.board
            .iter()
            .all(|row| row.iter().all(|cell| cell.is_some()))
    }

    pub fn column_full(&self, column: usize) -> bool {
        self.board[0][column].is_some()
    }

    fn check_winner(
        &self,
        row: usize,
        col: usize,
        row_delta: isize,
        col_delta: isize,
    ) -> Option<Player> {
        let cell = self.board.get(row).and_then(|row| row.get(col))?;
        if cell.is_none() {
            return None;
        }
        for i in 1..4 {
            let row = row.checked_add_signed(row_delta * i)?;
            let col = col.checked_add_signed(col_delta * i)?;
            if row >= BOARD_SIZE || col >= BOARD_SIZE {
                return None;
            }
            if self.board.get(row).and_then(|row| row.get(col)) != Some(cell) {
                return None;
            }
        }
        Some(cell.unwrap())
    }

    pub fn winner(&self) -> Option<Player> {
        for row in 0..BOARD_SIZE {
            if let Some(player) = self.check_winner(row, 0, 0, 1) {
                return Some(player);
            }
        }

        for col in 0..BOARD_SIZE {
            if let Some(player) = self.check_winner(0, col, 1, 0) {
                return Some(player);
            }
        }

        if let Some(player) = self.check_winner(0, 0, 1, 1) {
            return Some(player);
        }

        if let Some(player) = self.check_winner(0, BOARD_SIZE - 1, 1, -1) {
            return Some(player);
        }

        None
    }

    pub fn reset(&mut self) {
        self.board = [[Cell::default(); BOARD_SIZE]; BOARD_SIZE];
    }
}

impl Display for Connect4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.board {
            write!(f, "⬜")?;
            for cell in row {
                write!(
                    f,
                    "{}",
                    cell.map(|p| p.to_string()).unwrap_or('⬛'.to_string())
                )?;
            }
            writeln!(f, "⬜")?;
        }
        writeln!(f, "{}", "⬜".repeat(BOARD_SIZE + 2))?;
        if let Some(winner) = self.winner() {
            writeln!(f, "{} wins!", winner)?;
        } else if self.board_full() {
            writeln!(f, "No winner.")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Cell(Option<Player>);

impl Deref for Cell {
    type Target = Option<Player>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(player) => write!(f, "{}", player),
            None => write!(f, "⬛"),
        }
    }
}

impl From<Player> for Cell {
    fn from(player: Player) -> Self {
        Cell(Some(player))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Player {
    Milk,
    Cookie,
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Player::Milk => write!(f, "🥛"),
            Player::Cookie => write!(f, "🍪"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_gamestate {
        ($game:expr, $expected:expr) => {
            for (expected, line) in $expected.into_iter().zip($game.to_string().lines()) {
                assert_eq!(expected, line);
            }
        };
    }

    #[test]
    fn test_connect4() {
        let game = Connect4::new();
        let expected = vec![
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬜⬜⬜⬜⬜",
        ];
        assert_gamestate!(game, expected);
    }

    #[test]
    fn test_connect4_play() {
        let mut game = Connect4::new();
        let result = game.play(Player::Milk, 0);
        let expected = vec![
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜🥛⬛⬛⬛⬜",
            "⬜⬜⬜⬜⬜⬜",
        ];
        assert!(result.is_ok());
        assert_gamestate!(game, expected);
    }

    #[test]
    fn test_connect4_play_out_of_bounds() {
        let mut game = Connect4::new();
        let result = game.play(Player::Milk, 10);
        let expected = vec![
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬜⬜⬜⬜⬜",
        ];
        assert_gamestate!(game, expected);
        assert!(result.is_err());
    }

    #[test]
    fn test_connect4_play_full_column() {
        let mut game = Connect4::new();
        assert!(game.play(Player::Milk, 0).is_ok());
        assert!(game.play(Player::Cookie, 0).is_ok());
        assert!(game.play(Player::Milk, 0).is_ok());
        assert!(game.play(Player::Cookie, 0).is_ok());
        assert!(game.play(Player::Milk, 0).is_err());

        let expected = vec![
            "⬜🍪⬛⬛⬛⬜",
            "⬜🥛⬛⬛⬛⬜",
            "⬜🍪⬛⬛⬛⬜",
            "⬜🥛⬛⬛⬛⬜",
            "⬜⬜⬜⬜⬜⬜",
        ];
        assert_gamestate!(game, expected);
    }

    #[test]
    fn test_connect4_play_full_board() {
        let mut game = Connect4::new();
        for col in 0..BOARD_SIZE {
            for _ in 0..BOARD_SIZE {
                assert!(game.play(Player::Milk, col).is_ok());
            }
            assert!(game.play(Player::Milk, col).is_err());
        }
        let expected = vec![
            "⬜🥛🥛🥛🥛⬜",
            "⬜🥛🥛🥛🥛⬜",
            "⬜🥛🥛🥛🥛⬜",
            "⬜🥛🥛🥛🥛⬜",
            "⬜⬜⬜⬜⬜⬜",
        ];
        assert_gamestate!(game, expected);
    }

    #[test]
    fn test_connect4_no_winner() {
        let mut game = Connect4::new();
        let expected = vec![
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬜⬜⬜⬜⬜",
        ];
        assert_gamestate!(game, expected);
        assert_eq!(game.winner(), None);
    }

    #[test]
    fn test_connect4_winner_row_milk() {
        let mut game = Connect4::new();
        assert!(game.play(Player::Milk, 0).is_ok());
        assert!(game.play(Player::Milk, 1).is_ok());
        assert!(game.play(Player::Milk, 2).is_ok());
        assert!(game.play(Player::Milk, 3).is_ok());

        let expected = vec![
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜⬛⬛⬛⬛⬜",
            "⬜🥛🥛🥛🥛⬜",
            "⬜⬜⬜⬜⬜⬜",
            "🥛 wins!",
        ];
        assert_gamestate!(game, expected);
        assert_eq!(game.winner(), Some(Player::Milk));
    }

    #[test]
    fn test_connect4_winner_column_cookie() {
        let mut game = Connect4::new();
        assert!(game.play(Player::Cookie, 0).is_ok());
        assert!(game.play(Player::Cookie, 0).is_ok());
        assert!(game.play(Player::Cookie, 0).is_ok());
        assert!(game.play(Player::Cookie, 0).is_ok());

        let expected = vec![
            "⬜🍪⬛⬛⬛⬜",
            "⬜🍪⬛⬛⬛⬜",
            "⬜🍪⬛⬛⬛⬜",
            "⬜🍪⬛⬛⬛⬜",
            "⬜⬜⬜⬜⬜⬜",
            "🍪 wins!",
        ];
        assert_gamestate!(game, expected);
        assert_eq!(game.winner(), Some(Player::Cookie));
    }

    #[test]
    fn test_connect4_winner_diagonal_cookie() {
        let mut game = Connect4::new();
        assert!(game.play(Player::Cookie, 0).is_ok());

        assert!(game.play(Player::Milk, 1).is_ok());
        assert!(game.play(Player::Cookie, 1).is_ok());

        assert!(game.play(Player::Cookie, 2).is_ok());
        assert!(game.play(Player::Milk, 2).is_ok());
        assert!(game.play(Player::Cookie, 2).is_ok());

        assert!(game.play(Player::Milk, 3).is_ok());
        assert!(game.play(Player::Cookie, 3).is_ok());
        assert!(game.play(Player::Milk, 3).is_ok());
        assert!(game.play(Player::Cookie, 3).is_ok());

        let expected = vec![
            "⬜⬛⬛⬛🍪⬜",
            "⬜⬛⬛🍪🥛⬜",
            "⬜⬛🍪🥛🍪⬜",
            "⬜🍪🥛🍪🥛⬜",
            "⬜⬜⬜⬜⬜⬜",
            "🍪 wins!",
        ];
        assert_gamestate!(game, expected);

        assert_eq!(game.winner(), Some(Player::Cookie));
    }

    #[test]
    fn test_connect4_winner_diagonal_milk() {
        let mut game = Connect4::new();
        assert!(game.play(Player::Cookie, 0).is_ok());
        assert!(game.play(Player::Milk, 0).is_ok());
        assert!(game.play(Player::Cookie, 0).is_ok());
        assert!(game.play(Player::Milk, 0).is_ok());

        assert!(game.play(Player::Milk, 1).is_ok());
        assert!(game.play(Player::Cookie, 1).is_ok());
        assert!(game.play(Player::Milk, 1).is_ok());

        assert!(game.play(Player::Cookie, 2).is_ok());
        assert!(game.play(Player::Milk, 2).is_ok());

        assert!(game.play(Player::Milk, 3).is_ok());

        let expected = vec![
            "⬜🥛⬛⬛⬛⬜",
            "⬜🍪🥛⬛⬛⬜",
            "⬜🥛🍪🥛⬛⬜",
            "⬜🍪🥛🍪🥛⬜",
            "⬜⬜⬜⬜⬜⬜",
            "🥛 wins!",
        ];
        assert_gamestate!(game, expected);

        assert_eq!(game.winner(), Some(Player::Milk));
    }

    #[test]
    fn test_full_board_no_winner() {
        let mut game = Connect4::new();

        // Fill board in a pattern that prevents any 4-in-a-row
        assert!(game.play(Player::Milk, 0).is_ok());
        assert!(game.play(Player::Milk, 0).is_ok());
        assert!(game.play(Player::Cookie, 0).is_ok());
        assert!(game.play(Player::Cookie, 0).is_ok());

        assert!(game.play(Player::Cookie, 1).is_ok());
        assert!(game.play(Player::Cookie, 1).is_ok());
        assert!(game.play(Player::Milk, 1).is_ok());
        assert!(game.play(Player::Milk, 1).is_ok());

        assert!(game.play(Player::Milk, 2).is_ok());
        assert!(game.play(Player::Milk, 2).is_ok());
        assert!(game.play(Player::Cookie, 2).is_ok());
        assert!(game.play(Player::Cookie, 2).is_ok());

        assert!(game.play(Player::Milk, 3).is_ok());
        assert!(game.play(Player::Milk, 3).is_ok());
        assert!(game.play(Player::Cookie, 3).is_ok());
        assert!(game.play(Player::Cookie, 3).is_ok());

        dbg!(&game.to_string());
        let expected = vec![
            "⬜🍪🥛🍪🍪⬜",
            "⬜🍪🥛🍪🍪⬜",
            "⬜🥛🍪🥛🥛⬜",
            "⬜🥛🍪🥛🥛⬜",
            "⬜⬜⬜⬜⬜⬜",
            "No winner.",
        ];
        assert_gamestate!(game, expected);
        assert_eq!(game.winner(), None);
    }
}

use std::fmt::Display;

use crate::{
    Board, BoardLayout, BoardTile, Coordinate, Direction, Hand, Square, Tile, WordPlacementError,
};

pub struct Player {
    name: String,
    score: isize,
    hand: Hand,
}

impl Player {
    pub fn new(name: String) -> Self {
        Player {
            name,
            score: 0,
            hand: Hand::empty(),
        }
    }
}

pub enum GameState {
    /// Turn of a player referenced by index
    Turn(usize),
    Done,
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameState::Turn(n) => write!(f, "Turn {n}"),
            GameState::Done => write!(f, "Done"),
        }
    }
}

pub struct Game {
    board: Board,
    players: Vec<Player>,
    state: GameState,
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "State: {}, Players: ", &self.state).unwrap();
        for Player { hand, score, name } in &self.players {
            writeln!(f, "Player {name}, (score: {score}), hand: {hand}")?;
        }

        write!(f, "{}", self.board)?;

        Ok(())
    }
}

impl Game {
    pub fn new(players: Vec<Player>, board_layout: BoardLayout) -> Self {
        assert!(!players.is_empty());
        assert!(players.len() <= 4);

        // TODO: Probably put some tiles into the players hands

        Self {
            board: Board::from(board_layout),
            state: GameState::Turn(0),
            players,
        }
    }

    pub fn place_tile(&mut self, tile: Tile, coord: Coordinate) -> Result<(), WordPlacementError> {
        // TODO: Probably check if the player has the required tiles, and modify the players accordingly

        self.board.place_tile(tile, coord)
    }

    pub fn get_tile(&self, coord: Coordinate) -> Option<BoardTile> {
        self.board.get_tile(coord)
    }

    pub fn end_turn(&mut self) -> Result<(), WordPlacementError> {
        // TODO: Refactor this function
        // - Things which only concern the board should be lowered into functions on Board
        // - Draw new tiles for the player

        let mut tile_iter = self
            .board
            .tiles_with_coordinates()
            .filter_map(|(coord, tile)| match tile {
                Some(tile) if tile.is_provisional == true => Some((coord, tile)),
                _ => None,
            });

        // check that we have played at least a tile
        let Some((first_coord, _)) = tile_iter.next() else {
            return Err(WordPlacementError::PlayedWordEmpty);
        };

        // check that we have played tiles in a (straight) line
        let mut axes = (Some(first_coord.x), Some(first_coord.y));
        for (c, _) in tile_iter {
            if Some(c.x) != axes.0 {
                axes.0 = None;
            }
            if Some(c.y) != axes.1 {
                axes.1 = None;
            }
        }

        // select direction for gap check
        let dir = match axes {
            (Some(_), _) => Direction::Vertical,
            (_, Some(_)) => Direction::Horizontal,
            (None, None) => return Err(WordPlacementError::InvalidDirection),
        };

        let coords_vec: Vec<Coordinate> = self.board.find_range(first_coord, dir).collect();

        // check that there are no provisional tiles in the board
        // that aren't in this range
        let all_tiles = self
            .board
            .tiles_with_coordinates()
            .filter(|(_, t)| t.is_some_and(|t| t.is_provisional));
        for (coord, tile) in all_tiles {
            if !coords_vec.contains(&coord) {
                println!("{coord:?}");
                println!("{tile:?}");

                return Err(WordPlacementError::ScatteredProvisionalTile);
            }
        }

        // check that there are unprovisional tiles in the range
        // if not, then check that the range is adjacent to at least one
        // non-provisional tile
        if self
            .board
            .find_range(first_coord, dir)
            .all(|coord| self.get_tile(coord).unwrap().is_provisional)
        {
            let other_dir = dir.flip();
            let mut is_adjacent = false;

            // for each adjacent tile in the other direction check that it's a valid word
            for position in coords_vec {
                let range = self
                    .board
                    .find_range(position, other_dir)
                    .map(|coord| self.get_tile(coord).unwrap());
                let range_vec: Vec<BoardTile> = range.collect();

                if range_vec.len() > 1 {
                    is_adjacent = true;
                }
            }

            if !is_adjacent {
                if !self.board.find_range(first_coord, dir).any(|coordinate| {
                    self.board
                        .get_square(coordinate)
                        .unwrap()
                        .eq(&Square::CenterSquare)
                }) {
                    return Err(WordPlacementError::WordNotAdjacent);
                }
            }
        }

        for coord in self.board.find_range(first_coord, dir) {
            self.board
                .get_tile_mut(coord)
                .unwrap()
                .as_mut()
                .unwrap()
                .is_provisional = false;
        }

        Ok(())
    }
}

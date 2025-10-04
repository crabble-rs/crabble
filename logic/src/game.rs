use std::fmt::Display;

use crate::{
    bag::Bag, language::Language, Board, BoardLayout, BoardTile, Coordinate, CrabbleError,
    Direction, Hand, HandTile, Square, Tile,
};

#[derive(Debug)]
pub struct Player {
    name: String,
    score: isize,
    pub hand: Hand,
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

#[derive(Debug)]
pub enum GameState {
    /// Turn of a player referenced by index
    Turn(usize, bool),
    Done,
}

impl Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameState::Turn(n, _is_last_round) => write!(f, "Turn {n}"),
            GameState::Done => write!(f, "Done"),
        }
    }
}

impl Player {
    pub fn draw_from_bag(&mut self, bag: &mut Bag) -> Result<(), CrabbleError> {
        while self.hand.letters.len() < 7 {
            let drawn_tile = bag.take();
            match drawn_tile {
                Some(x) => self.hand.letters.push(x),
                None => return Err(CrabbleError::BagEmpty),
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Game {
    board: Board,
    bag: Bag,
    pub players: Vec<Player>,
    pub state: GameState,
    language: Language,
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
    pub fn new(mut players: Vec<Player>, board_layout: BoardLayout, language: Language) -> Self {
        assert!(!players.is_empty());
        assert!(players.len() >= 1);
        assert!(players.len() <= 4);

        let mut bag = Bag::full(&language.distribution);
        bag.shuffle();

        for player in &mut players {
            player.draw_from_bag(&mut bag).unwrap();
            assert_eq!(player.hand.letters.len(), 7);
        }

        Self {
            board: Board::from(board_layout),
            bag,
            state: GameState::Turn(0, false),
            language,
            players,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn get_tile(&self, coord: Coordinate) -> Option<BoardTile> {
        self.board.get_tile(coord)
    }

    pub fn display_current_player_hand(&self) -> String {
        let mut res = String::new();

        if let GameState::Turn(t, _is_last_round) = self.state {
            let current_player_hand = &self.players.get(t).unwrap().hand;
            for letter in &current_player_hand.letters {
                let score = self.language.values.get(*letter);
                let tile = format!("({}, {}) ", letter.to_string(), score);
                res.push_str(&tile);
            }
        }
        res
    }

    pub fn place_tile(&mut self, tile: Tile, coord: Coordinate) -> Result<(), CrabbleError> {
        // is_provisionary is true
        // we place the tiles on
        let board_tile = self
            .board
            .get_tile_mut(coord)
            .ok_or(CrabbleError::TileOutOufBounds)?;

        match board_tile {
            Some(_) => Err(CrabbleError::TileOccupied),
            None => {
                *board_tile = Some(BoardTile {
                    tile,
                    is_provisional: true,
                });
                self.board.provisionary_tiles_count += 1;
                Ok(())
            }
        }
    }

    pub fn end_turn(&mut self) -> Result<(), CrabbleError> {
        let mut tile_iter = self
            .board
            .tiles_with_coordinates()
            .filter_map(|(coord, tile)| match tile {
                Some(tile) if tile.is_provisional == true => Some((coord, tile)),
                _ => None,
            });

        // check that we have played at least a tile
        let Some((first_coord, _)) = tile_iter.next() else {
            return Err(CrabbleError::PlayedWordEmpty);
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
            (Some(_), Some(_)) => {
                let h = self
                    .board
                    .find_range(first_coord, Direction::Horizontal)
                    .collect::<Vec<_>>();
                let v = self
                    .board
                    .find_range(first_coord, Direction::Vertical)
                    .collect::<Vec<_>>();
                if h.len() > v.len() {
                    Direction::Horizontal
                } else {
                    Direction::Vertical
                }
            }
            (Some(_), _) => Direction::Vertical,
            (_, Some(_)) => Direction::Horizontal,
            (None, None) => return Err(CrabbleError::InvalidDirection),
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

                return Err(CrabbleError::ScatteredProvisionalTile);
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
                    return Err(CrabbleError::WordNotAdjacent);
                }
            }
        }

        let score = self.score_word(self.board.find_range(first_coord, dir), dir);

        match self.state {
            GameState::Turn(n, is_last_round) => {
                let player_id = n % self.players.len();
                let current_player = self.players.get_mut(player_id).unwrap();
                current_player.score += score;

                let _ = current_player.draw_from_bag(&mut self.bag);

                if self.bag.is_empty() && current_player.hand.is_empty() {
                    // starting final round
                    self.state = GameState::Turn(n + 1, true);
                } else {
                    self.state = GameState::Turn(n + 1, is_last_round);
                }

                match self.state {
                    GameState::Turn(player, is_last_round) => {
                        if is_last_round && self.players.get(player).unwrap().hand.is_empty() {
                            self.state = GameState::Done;
                            return Ok(());
                        }
                    }
                    GameState::Done => return Ok(()),
                }
            }
            GameState::Done => return Ok(()),
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

    fn score_word(&self, word: impl Iterator<Item = Coordinate>, dir: Direction) -> isize {
        // TODO: count scrabbles
        let other_dir = dir.flip();
        let mut total = 0;

        let word_vec: Vec<_> = word.collect();

        total += self.score_range(word_vec.iter().cloned(), false);

        for tile in word_vec {
            let range = self.board.find_range(tile, other_dir);
            let range_vec: Vec<_> = range.collect();
            if range_vec.len() > 1 {
                total += self.score_range(range_vec.iter().cloned(), true);
            }
        }

        total
    }

    fn score_range(&self, word: impl Iterator<Item = Coordinate>, is_adjacent_word: bool) -> isize {
        let mut total: isize = 0;
        let mut word_multiplier = 1;

        for letter in word {
            let tile = self
                .board
                .get_tile(letter)
                .map(|tile| {
                    let t = tile.tile;
                    match t.is_joker {
                        true => HandTile::Joker,
                        false => HandTile::Letter(t.tile),
                    }
                })
                .unwrap();

            let mut value = self.language.values.get(tile) as isize;

            if !is_adjacent_word && self.board.get_tile(letter).unwrap().is_provisional {
                match self.board.get_square(letter).unwrap() {
                    Square::Empty => (),
                    Square::CenterSquare => word_multiplier = word_multiplier * 2,
                    Square::LetterMultiplier(m) => value = value * (m as isize),
                    Square::WordMultiplier(m) => word_multiplier = word_multiplier * (m as isize),
                };
            }

            total += value;
        }
        total * word_multiplier
    }
}

#[cfg(test)]
mod tests {
    use crate::asn::ASN;
    use std::str::FromStr;

    use crate::game::*;

    #[test]
    fn scoring_test_1() {
        let layout = BoardLayout::from_fn((15, 15), crate::standard_board_layout);

        let players = vec![
            Player::new("Gamer 1".to_string()),
            Player::new("Player 2".to_string()),
        ];

        let mut game = Game::new(players, layout, Language::by_name("english").unwrap());

        let a = ASN::from_str("77hcat\na7hs").unwrap();
        a.run(&mut game, true).unwrap();

        let scores: Vec<_> = game.players.iter().map(|p| p.score).collect();
        assert_eq!(scores, [10, 6])
    }
}

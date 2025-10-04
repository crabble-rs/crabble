use std::{io::Read, path::PathBuf, str::FromStr};

use crate::game::Game;

use crate::{Coordinate, CrabbleError, Direction, Tile};

pub struct ASN {
    pub lines: Vec<ASNLine>,
}

impl ASN {
    pub fn from_file(file: PathBuf) -> Self {
        let mut s = String::new();
        std::fs::File::open(file)
            .expect("failed to open file")
            .read_to_string(&mut s)
            .unwrap();

        ASN::from_str(&s).unwrap()
    }

    pub fn run(self, game: &mut Game, print_board: bool) -> Result<(), CrabbleError> {
        for line in self.lines {
            let mut coord = line.coord;

            for tile in line.tiles {
                game.place_tile(tile, coord)?;

                while game.get_tile(coord).is_some() {
                    coord += line.dir.to_offset();
                }
            }
            if print_board {
                println!("{game}");
            }
            game.end_turn()?;
        }

        Ok(())
    }
}

pub struct ASNLine {
    pub coord: Coordinate,
    pub dir: Direction,
    pub tiles: Vec<Tile>,
}

#[derive(Debug)]
pub enum ASNError {
    InvalidCoord,
    InvalidDirection,
    InvalidTileCharacter,
    InvalidJoker,
    UnexpendedPlayEnd,
}

impl FromStr for ASN {
    type Err = ASNError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut asn_lines = vec![];
        let mut chars = s.chars();

        while let Some(c) = chars.next() {
            let x =
                isize::from_str_radix(&c.to_string(), 15).map_err(|_| ASNError::InvalidCoord)?;
            let y = match chars.next() {
                Some(c) => {
                    isize::from_str_radix(&c.to_string(), 15).map_err(|_| ASNError::InvalidCoord)?
                }
                _ => return Err(ASNError::InvalidCoord),
            };

            let dir = match chars.next() {
                Some('v') => Direction::Vertical,
                Some('h') => Direction::Horizontal,
                _ => return Err(ASNError::InvalidDirection),
            };

            #[derive(Copy, Clone, Debug)]
            enum ParseState {
                JokerTile,
                RequiresTile,
                CanEnd,
            }
            use ParseState::*;

            let mut state = RequiresTile;
            let mut tiles = vec![];
            loop {
                match (state, chars.next()) {
                    (RequiresTile | CanEnd, Some('*')) => state = ParseState::JokerTile,
                    (JokerTile, Some('*')) => return Err(ASNError::InvalidJoker),

                    (CanEnd, None | Some('\n')) => break,
                    (JokerTile | RequiresTile, None | Some('\n')) => {
                        return Err(ASNError::UnexpendedPlayEnd)
                    }

                    (_, Some(c)) if c.is_alphabetic() => {
                        tiles.push(Tile {
                            tile: c,
                            is_joker: matches!(state, JokerTile),
                        });
                        state = CanEnd;
                    }
                    _ => return Err(ASNError::InvalidTileCharacter),
                }
            }

            asn_lines.push(ASNLine {
                coord: Coordinate { x, y },
                dir,
                tiles,
            });
        }

        Ok(ASN { lines: asn_lines })
    }
}

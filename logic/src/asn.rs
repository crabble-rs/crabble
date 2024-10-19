// Algebraic Scrabble Notation
// 98hCA*TS

use std::str::FromStr;

use crate::{BoardTile, Coordinate, Direction, Tile};

struct ASN {
    coord: Coordinate,
    dir: Direction,
    tiles: BoardTile,
}

enum ASNError {
    InvalidCoord,
    InvalidDirection,
    InvalidTileCharacter
}

impl FromStr for ASN {
    type Err = ASNError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let x = match chars.next() {
            Some(c) if c < 'g' => isize::from_str_radix(&c.to_string(), 15),
            _ => return Err(ASNError::InvalidCoord),
        };
        let y = match chars.next() {
            Some(c) if c < 'g' => isize::from_str_radix(&c.to_string(), 15),
            _ => return Err(ASNError::InvalidCoord),
        };
        let dir = match chars.next() {
            Some('v') => Direction::Vertical,
            Some('h') => Direction::Horizontal,
            _ => return Err(ASNError::InvalidDirection),
        };

        match chars.next() {
            Some('*') => {
                let next_char = chars.next();
                // let tile = match next_char {
                //     Some(c) if c.is_alphabetic() => Tile {
                //         tile: c,
                //         is_joker: true,
                //     },
                //     None => return Err(ASNError::InvalidTileCharacter)
                }
            },
            Some(char) 
        }
        todo!();
    }
}

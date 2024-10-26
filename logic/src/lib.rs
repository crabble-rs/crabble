#![allow(dead_code)]

pub mod asn;
mod bag;
mod game;
mod language;

use std::fmt::Display;
use std::ops::{Add, AddAssign, Sub};
use std::sync::LazyLock;

#[derive(Clone, Debug)]
pub struct BoardLayout {
    squares: Vec<Vec<Square>>,
}

#[derive(Debug, PartialEq)]
pub enum WordPlacementError {
    TileOccupied,
    PlayedWordEmpty,
    InvalidDirection,
    ScatteredProvisionalTile,
    WordNotAdjacent,
    TileOutOufBounds,
}

impl BoardLayout {
    fn get(&self, index: Coordinate) -> Option<Square> {
        let x_checked: usize = index.x.try_into().ok()?;
        let y_checked: usize = index.y.try_into().ok()?;
        let column = self.squares.get(x_checked)?;
        column.get(y_checked).cloned()
    }

    fn dimensions(&self) -> (usize, usize) {
        (
            self.squares.len(),
            self.squares.first().map_or(0, |v| v.len()),
        )
    }

    fn from_fn(dimensions: (usize, usize), f: impl Fn(Coordinate) -> Square) -> Self {
        let mut squares = vec![vec![Square::Empty; dimensions.1]; dimensions.0];

        for (x, col) in squares.iter_mut().enumerate() {
            for (y, square) in col.iter_mut().enumerate() {
                *square = f(Coordinate {
                    x: x as isize,
                    y: y as isize,
                });
            }
        }

        Self { squares }
    }
}

fn standard_board_layout(Coordinate { mut x, mut y }: Coordinate) -> Square {
    if x == 7 && y == 7 {
        return Square::CenterSquare;
    }
    if x % 7 == 0 && y % 7 == 0 {
        return Square::WordMultiplier(3);
    }
    if x > 7 {
        x = 14 - x;
    }
    if y > 7 {
        y = 14 - y;
    }
    if y > x {
        (x, y) = (y, x)
    }

    match (x, y) {
        (3, 0) => Square::LetterMultiplier(2),
        (5, 1) => Square::LetterMultiplier(3),
        (6, 2) => Square::LetterMultiplier(2),
        (7, 3) => Square::LetterMultiplier(2),
        (6, 6) => Square::LetterMultiplier(2),
        (5, 5) => Square::LetterMultiplier(3),
        (x, y) if x == y => Square::WordMultiplier(2),
        _ => Square::Empty,
    }
}

impl Display for BoardLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x_max, y_max) = self.dimensions();
        for x in 0..x_max {
            for y in 0..y_max {
                let s = self
                    .get(Coordinate {
                        x: x as isize,
                        y: y as isize,
                    })
                    .unwrap();

                write!(
                    f,
                    "{}",
                    char::from_u32(match s {
                        Square::Empty => b'.',
                        Square::CenterSquare => b'*',
                        Square::LetterMultiplier(x) => x as u8 + b'0',
                        Square::WordMultiplier(x) => x as u8 + b' ',
                    } as u32)
                    .unwrap()
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (x_max, y_max) = self.layout.dimensions();
        for y in 0..y_max {
            for x in 0..x_max {
                let s = self
                    .layout
                    .get(Coordinate {
                        x: x as isize,
                        y: y as isize,
                    })
                    .unwrap();
                let opt_t = self.get_tile(Coordinate {
                    x: x as isize,
                    y: y as isize,
                });
                match opt_t {
                    Some(tile) => {
                        if tile.is_provisional {
                            write!(f, "\x1b[35m").unwrap();
                        }
                        write!(f, "{}", tile.tile.tile).unwrap();
                        if tile.is_provisional {
                            write!(f, "\x1b[0m").unwrap();
                        }
                    }
                    None => {
                        write!(
                            f,
                            "{}",
                            char::from_u32(match s {
                                Square::Empty => b'.',
                                Square::CenterSquare => b'*',
                                Square::LetterMultiplier(x) => x as u8 + b'0',
                                Square::WordMultiplier(x) => x as u8 + b' ',
                            } as u32)
                            .unwrap()
                        )?;
                    }
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Board {
    // the actual squares on the board
    layout: BoardLayout,
    // the played letters
    tiles: Vec<Vec<Option<BoardTile>>>,
    provisionary_tiles_count: usize,
}

impl From<BoardLayout> for Board {
    fn from(value: BoardLayout) -> Self {
        let (outer_size, inner_size) = value.dimensions();
        Self {
            layout: value,
            tiles: vec![vec![None; inner_size]; outer_size],
            provisionary_tiles_count: 0,
        }
    }
}

impl Board {
    pub fn new() -> Board {
        todo!()
    }

    fn get_square(&self, coord: Coordinate) -> Option<Square> {
        self.layout.get(coord)
    }

    fn get_tile_mut(&mut self, coord: Coordinate) -> Option<&mut Option<BoardTile>> {
        let x_checked: usize = coord.x.try_into().ok()?;
        let y_checked: usize = coord.y.try_into().ok()?;
        let column: &mut Vec<Option<BoardTile>> = self.tiles.get_mut(x_checked)?;
        column.get_mut(y_checked)
    }

    fn get_tile(&self, coord: Coordinate) -> Option<BoardTile> {
        let x_checked: usize = coord.x.try_into().ok()?;
        let y_checked: usize = coord.y.try_into().ok()?;
        let column: &Vec<Option<BoardTile>> = self.tiles.get(x_checked)?;
        *column.get(y_checked)?
    }

    fn tiles_with_coordinates(&self) -> impl Iterator<Item = (Coordinate, Option<BoardTile>)> + '_ {
        self.tiles.iter().enumerate().flat_map(|(x, vec)| {
            vec.iter().enumerate().map(move |(y, tile)| {
                (
                    Coordinate {
                        x: (x as isize),
                        y: (y as isize),
                    },
                    *tile,
                )
            })
        })
    }

    // given a board, a coordinate, and a direction
    // find the range of the first contiguous chunk of tiles on the board containing coord, in that direction
    pub fn find_range(
        &self,
        coord: Coordinate,
        dir: Direction,
    ) -> impl Iterator<Item = Coordinate> {
        let mut range_begin = coord.clone();
        let mut range_end = coord.clone();

        let offset = dir.to_offset();

        // if we're not at the end of the board, and if we haven't found an empty tile:
        loop {
            match self.get_tile(range_end + offset) {
                Some(_) => range_end = range_end + offset,
                None => break,
            }
        }

        // iterate the other way...
        loop {
            match self.get_tile(range_begin - offset) {
                Some(_) => range_begin = range_begin - offset,
                None => break,
            }
        }

        let mut current_coord = range_begin;
        std::iter::from_fn(move || match dir {
            Direction::Horizontal => {
                if current_coord.x > range_end.x {
                    return None;
                } else {
                    let res = Some(current_coord);
                    current_coord += offset;
                    return res;
                }
            }
            Direction::Vertical => {
                if current_coord.y > range_end.y {
                    return None;
                } else {
                    let res = Some(current_coord);
                    current_coord += offset;
                    return res;
                }
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coordinate {
    x: isize,
    y: isize,
}

impl Add for Coordinate {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Coordinate {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl AddAssign for Coordinate {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Coordinate {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Coordinate {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Coordinate {
    fn checked_sub(self, other: Self) -> Option<Self> {
        if other.x > self.x || other.y > self.y {
            None
        } else {
            Some(Coordinate {
                x: self.x - other.x,
                y: self.y - other.y,
            })
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct BoardTile {
    tile: Tile,
    is_provisional: bool,
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash)]
pub struct Tile {
    tile: char,
    is_joker: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum HandTile {
    Joker,
    Letter(char),
}

impl Display for HandTile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = match self {
            Self::Joker => '*',
            Self::Letter(l) => *l,
        };

        write!(f, "{ch}")
    }
}

#[derive(Debug)]
struct Hand {
    letters: Vec<HandTile>,
}

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hand = self
            .letters
            .iter()
            .map(|el| el.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "{}", hand)
    }
}

impl Hand {
    pub fn empty() -> Self {
        Hand {
            letters: Vec::new(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Square {
    Empty,
    CenterSquare,
    LetterMultiplier(i8),
    WordMultiplier(i8),
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    fn to_offset(self) -> Coordinate {
        match self {
            Self::Horizontal => Coordinate { x: 1, y: 0 },
            Self::Vertical => Coordinate { x: 0, y: 1 },
        }
    }

    fn flip(self) -> Direction {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

/// Implements whether a certain play is valid. Returns whether the play is valid.
fn challenge(
    board: &Board,
    word: impl Iterator<Item = Coordinate> + Clone,
    dir: Direction,
) -> bool {
    if !check_if_valid(word.clone().map(|coord| board.get_tile(coord).unwrap())) {
        return false;
    }
    let other_dir = dir.flip();
    for letter in word {
        let word = board
            .find_range(letter, other_dir)
            .map(|coord| board.get_tile(coord).unwrap());
        if !check_if_valid(word) {
            return false;
        }
    }

    true
}

fn check_if_valid(word: impl Iterator<Item = BoardTile>) -> bool {
    static WORDS: LazyLock<Vec<&str>> = LazyLock::new(|| {
        include_str!("../../data/english/words.txt")
            .lines()
            .collect()
    });

    let word: String = word.map(|w| w.tile.tile).collect();

    if word.is_empty() {
        return false;
    }

    WORDS.binary_search(&word.as_ref()).is_ok()
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use asn::ASN;

    use super::*;

    #[test]
    fn test_standard_board_layout() {
        let layout = BoardLayout::from_fn((15, 15), standard_board_layout);
        let s = layout.to_string();
        assert_eq!(s, include_str!("../../data/scrabble_layout.txt"),);
    }

    // #[test]
    // fn test_any_first_play() {
    //     // create default board layout
    //     let layout = BoardLayout::from_fn((15, 15), standard_board_layout);
    //     let mut board: Board = layout.into();

    //     board
    //         .place_tile(
    //             Tile {
    //                 tile: 'a',
    //                 is_joker: false,
    //             },
    //             Coordinate { x: 7, y: 7 },
    //         )
    //         .unwrap();
    //     board
    //         .place_tile(
    //             Tile {
    //                 tile: 'a',
    //                 is_joker: false,
    //             },
    //             Coordinate { x: 6, y: 7 },
    //         )
    //         .unwrap();
    //     board.end_turn().unwrap();
    // }

    // #[test]
    // fn test_word_extension() {
    //     let layout = BoardLayout::from_fn((15, 15), standard_board_layout);
    //     let mut board: Board = layout.into();

    //     board
    //         .place_tile(
    //             Tile {
    //                 tile: 'c',
    //                 is_joker: false,
    //             },
    //             Coordinate { x: 7, y: 7 },
    //         )
    //         .unwrap();
    //     board
    //         .place_tile(
    //             Tile {
    //                 tile: 'a',
    //                 is_joker: false,
    //             },
    //             Coordinate { x: 8, y: 7 },
    //         )
    //         .unwrap();
    //     board
    //         .place_tile(
    //             Tile {
    //                 tile: 't',
    //                 is_joker: false,
    //             },
    //             Coordinate { x: 9, y: 7 },
    //         )
    //         .unwrap();
    //     board.end_turn().unwrap();
    //     board
    //         .place_tile(
    //             Tile {
    //                 tile: 's',
    //                 is_joker: false,
    //             },
    //             Coordinate { x: 10, y: 7 },
    //         )
    //         .unwrap();
    //     board.end_turn().unwrap();
    // }

    #[test]
    fn asn_test_word_extension() {
        let a = ASN::from_str("77hcat\na7hs").unwrap();
        a.run(true).unwrap();
    }

    #[test]
    fn asn_invalid_play() {
        let a = ASN::from_str("77hcat\ne8hs").unwrap();
        let err = a.run(false).unwrap_err();
        assert_eq!(err, WordPlacementError::WordNotAdjacent);
    }

    #[test]
    fn asn_invalid_play_overlap() {
        let a = ASN::from_str("77hcat\n97hmeow").unwrap();
        let err = a.run(false).unwrap_err();
        assert_eq!(err, WordPlacementError::TileOccupied);
    }

    #[test]
    fn asn_catgirl_extension() {
        let a = ASN::from_str("77hgirl\n47hcats").unwrap();
        a.run(true).unwrap();
    }
}

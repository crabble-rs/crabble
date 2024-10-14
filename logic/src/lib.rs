#![allow(dead_code)]

mod bag;
mod game;
mod language;

use std::fmt::Display;
use std::ops::{Add, AddAssign, Index, Sub};

#[derive(Clone, Debug)]
pub struct BoardLayout {
    squares: Vec<Vec<Square>>,
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

#[derive(Clone)]
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

    fn tiles_with_coordinates<'a>(
        &'a self,
    ) -> impl Iterator<Item = (Coordinate, Option<BoardTile>)> + 'a {
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
}

#[derive(Clone, Copy)]
struct Coordinate {
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

#[derive(Clone, Copy)]
struct BoardTile {
    tile: Tile,
    is_provisional: bool,
}

#[derive(Clone, Copy)]
struct Tile {
    tile: char,
    is_joker: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum HandTile {
    Joker,
    Letter(char),
}

struct Hand {
    letters: Vec<HandTile>,
}

#[derive(Copy, Clone, Debug)]
pub enum Square {
    Empty,
    LetterMultiplier(i8),
    WordMultiplier(i8),
}

#[derive(Copy, Clone)]
pub enum Direction {
    Horizontal(usize),
    Vertical(usize),
}

fn place_tile(board: &mut Board, tile: Tile, coord: Coordinate) -> Result<(), ()> {
    // is_provisionary is true
    // we place the tiles on
    let board_tile = board.get_tile_mut(coord).ok_or(())?;

    match board_tile {
        Some(_) => Err(()),
        None => {
            *board_tile = Some(BoardTile {
                tile,
                is_provisional: true,
            });
            board.provisionary_tiles_count += 1;
            Ok(())
        }
    }
}

fn end_turn(board: &mut Board) -> Result<i32, ()> {
    let mut tile_iter = board
        .tiles_with_coordinates()
        .filter_map(|(coord, tile)| match tile {
            Some(tile) if tile.is_provisional == true => Some((coord, tile)),
            _ => None,
        });

    // check that we have played at least a tile
    let Some((first_coord, _)) = tile_iter.next() else {
        return Err(());
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
    let offset = match axes {
        (Some(_), _) => Coordinate { x: 0, y: 1 },
        (_, Some(_)) => Coordinate { x: 1, y: 0 },
        (None, None) => return Err(()),
    };

    let coords_vec: Vec<Coordinate> = find_range(board, first_coord, offset).collect();
    let tiles_iter: Vec<BoardTile> = find_range(board, first_coord, offset)
        .map(|coord| board.get_tile(coord))
        .collect();

    while current.x < board.layout.dimensions().0 && current.y < board.layout.dimensions().1 {
        if let Some(tile) = board.get_tile(current) {
            // return an error
            if tile.is_provisional {
                return Err(());
            }
        } else {
            // current tile is none, that means that we can continue checking row/column
            current = current + offset;
        }
    }

    check_if_valid(current_word);

    todo!()
}

// given a board, a coordinate, and a direction
// find the range of the first contiguous chunk of tiles on the board containing coord, in that direction
fn find_range(
    board: &Board,
    coord: Coordinate,
    offset: Coordinate,
) -> impl Iterator<Item = Coordinate> {
    let mut range_begin = coord.clone();
    let mut range_end = coord.clone();

    // if we're not at the end of the board, and if we haven't found an empty tile:
    loop {
        match board.get_tile(range_end) {
            Some(_) => range_end = range_end + offset,
            None => break,
        }
    }

    // iterate the other way...
    loop {
        match board.get_tile(range_begin) {
            Some(_) => range_begin = range_begin - offset,
            None => break,
        }
    }

    // create iterator from range
    let mut current_coord = range_begin;
    std::iter::from_fn(move || {
        if current_coord.x >= range_end.x || current_coord.y >= range_end.y {
            return None;
        }
        let res = Some(current_coord);
        current_coord += offset;
        res
    })
}

fn check_if_valid(word: impl Iterator<Item = BoardTile>) -> bool {
    true
}

#[cfg(test)]
mod test {
    use crate::{standard_board_layout, BoardLayout};

    #[test]
    fn test_standard_board_layout() {
        let layout = BoardLayout::from_fn((15, 15), standard_board_layout);
        let s = layout.to_string();
        assert_eq!(s, include_str!("../../data/scrabble_layout.txt"),);
    }
}

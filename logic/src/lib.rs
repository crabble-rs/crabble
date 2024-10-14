#![allow(dead_code)]

mod bag;

use std::ops::{Add, AddAssign, Sub};

#[derive(Clone)]
pub struct Board {
    // the actual squares on the board
    squares: Vec<Vec<Square>>,
    // the played letters
    tiles: Vec<Vec<Option<BoardTile>>>,
    provisionary_tiles_count: usize,
}

impl Board {
    pub fn new() -> Board {
        todo!()
    }

    fn get_square(&self, coord: Coordinate) -> &Square {
        &self.squares[coord.x][coord.y]
    }

    fn get_tile_mut(&mut self, coord: Coordinate) -> &mut Option<BoardTile> {
        &mut self.tiles[coord.x][coord.y]
    }

    fn get_tile(&self, coord: Coordinate) -> Option<BoardTile> {
        self.tiles[coord.x][coord.y]
    }

    fn tiles_with_coordinates<'a>(
        &'a self,
    ) -> impl Iterator<Item = (Coordinate, Option<BoardTile>)> + 'a {
        self.tiles.iter().enumerate().flat_map(|(x, vec)| {
            vec.iter()
                .enumerate()
                .map(move |(y, tile)| (Coordinate { x, y }, *tile))
        })
    }
}

#[derive(Clone, Copy)]
struct Coordinate {
    x: usize,
    y: usize,
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

#[derive(Copy, Clone)]
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
    let board_tile = board.get_tile_mut(coord);
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

    let mut current = Coordinate {
        x: first_coord.x,
        y: first_coord.y,
    };

    let mut current_tile = board.get_tile(current);

    while current_tile.is_some() {
        let next_tile = board.get_tile(current + offset);
        if next_tile.is_some() {
            // continue checking contiguous chunk
            current_tile = next_tile;
            current = current + offset;
        } else {
            break;
        }
    }

    let range_end = current.clone();

    let mut back = first_coord.clone();
    let mut current_tile = board.get_tile(back);

    while back.checked_sub(offset).is_some() && current_tile.is_some() {
        let next_tile = board.get_tile(back - offset);
        if next_tile.is_some() {
            // continue checking contiguous chunk
            current_tile = next_tile;
            back = back - offset;
        } else {
            break;
        }
    }

    let range_begin = back;

    while current.x < board.squares.len() && current.y < board.squares[0].len() {
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

    let mut current_coord = range_begin;

    let current_word = std::iter::from_fn(|| {
        current_coord += offset;

        if current_coord.x >= range_end.x || current_coord.y >= range_end.y {
            return None;
        }

        board.get_tile(current_coord)
    });

    check_if_valid(current_word);

    todo!()
}

fn check_if_valid(word: impl Iterator<Item = BoardTile>) -> bool {
    true
}

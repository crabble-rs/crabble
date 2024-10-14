use crate::{Board, BoardLayout, Hand};

struct Player {
    name: String,
    hand: Hand,
}

enum GameState {
    /// Turn of a player referenced by index
    Turn(usize),
    Done,
}

struct Game {
    board: Board,
    players: Vec<Player>,
    state: GameState,
}

impl Game {
    fn new(players: Vec<Player>, board_layout: BoardLayout) -> Self {
        assert!(!players.is_empty());
        Self {
            board: Board::from(board_layout),
            state: GameState::Turn(0),
            players,
        }
    }
}

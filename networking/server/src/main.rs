use std::collections::HashMap;

use logic::Board;
use store::Store;
use uuid::Uuid;

mod store;

#[derive(Clone)]
struct Player {
    name: String,
    id: Uuid,
}

#[derive(Clone, Copy)]
enum GameState {
    Pending,
    Playing(Uuid), // Uuid indicating which player's turn it is
    Completed
}

#[derive(Clone)]
struct Game {
    uuid: Uuid, // We're storing the Uuids de-normalized cause it makes it easier lol
    players: Vec<Player>, // Order determines turn order
    board: Board,
    state: GameState
}


fn main() {
    let store = HashMap::<Uuid, Game>::new();

    todo!("finish the rest lol")
}

fn start_game<S: Store>(store: &mut S) -> Result<Uuid, S::Error> {
    let uuid = Uuid::new_v4();

    let game = Game {
        uuid,
        players: Vec::new(),
        board: Board::new(),
        state: GameState::Pending
    };

    store.save_game(uuid, game)?;

    Ok(uuid)
}

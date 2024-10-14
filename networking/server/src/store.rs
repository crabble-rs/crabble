use std::collections::HashMap;

use uuid::Uuid;

use crate::Game;

pub trait Store {
    type Error;

    fn save_game(&mut self, uuid: Uuid, game: Game) -> Result<(), Self::Error>;
    fn load_game(&self, uuid: Uuid) -> Result<Game, Self::Error>;
    fn get_all_games(&self) -> Result<HashMap<Uuid, Game>, Self::Error>;

    // TODO: saving logins?
}

impl Store for HashMap<Uuid, Game> {
    type Error = ();

    fn save_game(&mut self, uuid: Uuid, game: Game) -> Result<(), Self::Error> {
        self.insert(uuid, game);
        Ok(())
    }

    fn load_game(&self, uuid: Uuid) -> Result<Game, Self::Error> {
        self.get(&uuid).ok_or(()).cloned()
    }

    fn get_all_games(&self) -> Result<HashMap<Uuid, Game>, Self::Error> {
        Ok(self.clone())
    }
}

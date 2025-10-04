use std::path::PathBuf;

use game::*;
use language::Language;
use logic::*;

fn main() {
    let mut directories = vec![PathBuf::from("./asn_tests")];

    while let Some(new_dir) = directories.pop() {
        for entry in std::fs::read_dir(&new_dir).unwrap() {
            let entry = entry.unwrap();
            let entry_type = entry.file_type().unwrap();

            if entry_type.is_dir() {
                directories.push(entry.path());
            }

            if entry_type.is_file() {
                let ext = entry.path();
                let ext = ext
                    .extension()
                    .map(|os_str| os_str.to_string_lossy().to_string())
                    .unwrap_or("".to_string());
                if ext != "asn" {
                    panic!("File in `asn_tests` with invalid extension: {:?}", ext);
                }

                println!("gaming!: {:?}", entry.path());

                let layout = BoardLayout::from_fn((15, 15), standard_board_layout);

                let players = vec![
                    Player::new("Gamer 1".to_string()),
                    Player::new("Player 2".to_string()),
                ];

                let mut game = Game::new(players, layout, Language::by_name("english").unwrap());

                let asn = logic::asn::ASN::from_file(entry.path());
                asn.run(&mut game, true).unwrap();
            }
        }
    }
}

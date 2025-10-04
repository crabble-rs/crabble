use std::str::FromStr;

use logic::asn::ASN;
use logic::game::{Game, GameState, Player};
use logic::language::Language;

use color_eyre::{Result, eyre};
use crossterm::event::{self, KeyCode, KeyEvent};
use logic::{BoardLayout, CrabbleError, standard_board_layout};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::{DefaultTerminal, Frame};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

/// App holds the state of the application
struct App {
    /// Current state of the TUI app
    state: State,
}

struct Settings {
    num_players: StringField,
    language: StringField,
    start_button: Button,
    active_box: SettingsActiveBox,
}

struct Button {
    text: String,
    selected: bool,
}

struct GameUI {
    active_box: GameTurnActiveBox,
    curr_board: StringField,
    curr_hand: StringField,
    curr_move: StringField,
    submit: Button,
}

enum GameTurnActiveBox {
    Move,
    Submit,
}

enum State {
    /// Prompts for game settings
    Setup(Settings),
    /// Current render of game state
    Gaming(
        // struct that holds everything that needs to be rendered as part of the current game turn
        // probably what we want here is some kind of textual representation of:
        // - the board
        // - current player's hand
        // - current player's desired move, in ASN
        // - some kind of submit button
        Game,
        GameUI,
    ),
}

enum SettingsActiveBox {
    NumPlayers,
    Language,
    Start,
}

/// A new-type representing a string field with a label.
#[derive(Debug)]

struct StringField {
    // whether this field is currently in focus
    selected: bool,
    /// Label above the field
    label: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Text currently in the box
    input: String,
}

impl GameUI {
    fn new(game: &Game) -> Self {
        let board = format!("{}", game.board());

        let curr_player = match game.state {
            GameState::Done => todo!(),
            GameState::Turn(n, _is_last_round) => n,
        };

        GameUI {
            active_box: GameTurnActiveBox::Move,
            curr_board: {
                let mut field =
                    StringField::new(format!("Current Board - Player {}'s turn", curr_player + 1));
                field.input = board;
                field
            },
            curr_hand: {
                let mut field = StringField::new("Current Player's hand".to_owned());
                field.input = game.display_current_player_hand();
                field
            },
            curr_move: {
                let mut field = StringField::new("Move".to_owned());
                field.selected = true;
                field
            },
            submit: Button::new("Submit Move".to_owned()),
        }
    }

    fn get_active_box(&mut self) -> Option<&mut StringField> {
        match self.active_box {
            GameTurnActiveBox::Move => Some(&mut self.curr_move),
            GameTurnActiveBox::Submit => None,
        }
    }
}

impl StringField {
    fn new(label: String) -> Self {
        StringField {
            selected: false,
            label,
            character_index: 0,
            input: String::new(),
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }
}

impl Widget for &Button {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let input = Paragraph::new(self.text.as_str()).style(match self.selected {
            true => Style::default(),
            false => Style::default().fg(Color::Yellow),
        });

        input.render(area, buf);
    }
}

impl Button {
    fn new(text: String) -> Self {
        Button {
            selected: false,
            text,
        }
    }
}

impl Widget for &StringField {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let input = Paragraph::new(self.input.as_str())
            .style(match self.selected {
                true => Style::default(),
                false => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title(self.label.clone()));

        input.render(area, buf);
    }
}

impl Settings {
    fn new() -> Self {
        Settings {
            num_players: StringField::new("How many players are there?".to_owned()),
            language: StringField::new("What language would you like to play in?".to_owned()),
            start_button: Button::new("Start Game!".to_owned()),
            active_box: SettingsActiveBox::NumPlayers,
        }
    }

    fn get_active_input_field(&mut self) -> Option<&mut StringField> {
        match self.active_box {
            SettingsActiveBox::NumPlayers => Some(&mut self.num_players),
            SettingsActiveBox::Language => Some(&mut self.language),
            SettingsActiveBox::Start => None,
        }
    }

    fn select_next_box(&mut self) {
        match self.active_box {
            SettingsActiveBox::NumPlayers => {
                self.active_box = SettingsActiveBox::Language;
                self.num_players.selected = false;
                self.language.selected = true;
                self.start_button.selected = false;
            }
            SettingsActiveBox::Language => {
                self.active_box = SettingsActiveBox::Start;
                self.num_players.selected = false;
                self.language.selected = false;
                self.start_button.selected = true;
            }
            SettingsActiveBox::Start => {
                self.active_box = SettingsActiveBox::NumPlayers;
                self.num_players.selected = true;
                self.language.selected = false;
                self.start_button.selected = false;
            }
        }
    }

    fn on_key_press(&mut self, event: KeyEvent) {
        if let KeyCode::Tab = event.code {
            self.select_next_box()
        }

        match self.active_box {
            SettingsActiveBox::NumPlayers | SettingsActiveBox::Language => {
                let active_box = self.get_active_input_field().unwrap();

                match event.code {
                    KeyCode::Char(c) => active_box.enter_char(c),
                    KeyCode::Backspace => active_box.delete_char(),
                    KeyCode::Left => active_box.move_cursor_left(),
                    KeyCode::Right => active_box.move_cursor_right(),
                    _ => {}
                }
            }
            SettingsActiveBox::Start => (),
        }
    }

    fn render(&self, frame: &mut Frame) {
        let [num_players_area, language_area, start_area] =
            Layout::vertical(Constraint::from_lengths([3, 3, 1])).areas(frame.area());

        frame.render_widget(&self.num_players, num_players_area);
        frame.render_widget(&self.language, language_area);

        frame.render_widget(&self.start_button, start_area);

        let (active_area, active_offset) = match self.active_box {
            SettingsActiveBox::Language => (language_area, self.language.character_index),
            SettingsActiveBox::NumPlayers => (num_players_area, self.num_players.character_index),
            SettingsActiveBox::Start => (start_area, 0),
        };
        if active_area != start_area {
            let cursor_pos =
                Position::new(active_area.x + active_offset as u16 + 1, active_area.y + 1);
            frame.set_cursor_position(cursor_pos);
        }
    }

    fn start_game(&mut self) -> Result<(Game, GameUI), CrabbleError> {
        let num_players = self
            .num_players
            .input
            .parse::<u32>()
            .map_err(|_| CrabbleError::InvalidNumberPlayers)?;
        let language = Language::by_name(&self.language.input).map_err(|e| e)?;

        let mut players = Vec::new();
        for i in 0..num_players {
            let player = Player::new(format!("player {}", i));
            players.push(player);
        }

        let layout = BoardLayout::from_fn((15, 15), standard_board_layout);

        let game = Game::new(players, layout, language);
        let ui = GameUI::new(&game);
        Ok((game, ui))
    }
}

impl App {
    fn new() -> Self {
        Self {
            state: State::Setup(Settings::new()),
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Some(key) = event::read()?.as_key_press_event() {
            self.on_key_press(key);
        }
        Ok(())
    }

    fn on_key_press(&mut self, event: KeyEvent) {
        match &mut self.state {
            State::Setup(settings) => match event.code {
                KeyCode::Enter => match settings.active_box {
                    SettingsActiveBox::Start => match settings.start_game() {
                        Ok((game, ui)) => self.state = State::Gaming(game, ui),
                        Err(e) => settings
                            .start_button
                            .text
                            .push_str(&format!(" - {}", e.to_string())),
                    },
                    _ => {}
                },
                _ => settings.on_key_press(event),
            },
            State::Gaming(game, game_turn) => {
                match event.code {
                    KeyCode::Enter => match game_turn.active_box {
                        GameTurnActiveBox::Move => {
                            game_turn.active_box = GameTurnActiveBox::Submit;
                            game_turn.curr_move.selected = false;
                            game_turn.submit.selected = true;
                        }
                        GameTurnActiveBox::Submit => {
                            let asn = ASN::from_str(&game_turn.curr_move.input).unwrap();
                            // `asn.run`` implicitly calls `end_turn`
                            asn.run(game, false).unwrap();
                            *game_turn = GameUI::new(game);
                        }
                    },
                    KeyCode::Tab => match game_turn.active_box {
                        GameTurnActiveBox::Move => {
                            game_turn.active_box = GameTurnActiveBox::Submit;
                            game_turn.curr_move.selected = false;
                            game_turn.submit.selected = true;
                        }
                        GameTurnActiveBox::Submit => {
                            game_turn.active_box = GameTurnActiveBox::Move;
                            game_turn.curr_move.selected = true;
                            game_turn.submit.selected = false;
                        }
                    },
                    KeyCode::Char(c) => game_turn.get_active_box().unwrap().enter_char(c),
                    KeyCode::Backspace => game_turn.get_active_box().unwrap().delete_char(),
                    KeyCode::Left => game_turn.get_active_box().unwrap().move_cursor_left(),
                    KeyCode::Right => game_turn.get_active_box().unwrap().move_cursor_right(),
                    _ => {}
                };
            }
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
    }

    fn render(&self, frame: &mut Frame) {
        match &self.state {
            State::Setup(settings) => settings.render(frame),
            State::Gaming(_, _) => self.render_game_state(frame),
        }
    }

    fn render_game_state(&self, frame: &mut Frame) {
        let State::Gaming(_, game_ui) = &self.state else {
            panic!("wrong state")
        };

        let [cur_board, cur_hand, cur_move, button] =
            Layout::vertical(Constraint::from_lengths([17, 3, 3, 1])).areas(frame.area());

        frame.render_widget(&game_ui.curr_board, cur_board);
        frame.render_widget(&game_ui.curr_hand, cur_hand);
        frame.render_widget(&game_ui.curr_move, cur_move);
        frame.render_widget(&game_ui.submit, button);
    }
}

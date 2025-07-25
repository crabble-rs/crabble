use core::num;

// TODO: get rid of UB lol ???
use logic::game::{self, Game, Player};
use logic::language::Language;

use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use logic::{BoardLayout, standard_board_layout};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, List, ListItem, Paragraph, Widget};
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
    /// Prompts for game settings
    settings: Settings,
    /// Current render of game state
    game_state: Option<GameTurn>,
}

struct Settings {
    num_players: StringField,
    language: StringField,
    start_button: Button,
    active_box: ActiveBox,
}

struct Button {
    text: String,
    selected: bool,
}

struct GameTurn {
    // struct that holds everything that needs to be rendered as part of the current game turn
    // probably what we want here is some kind of textual representation of:
    // - the board
    // - current player's hand
    // - current player's desired move, in ASN
    // - some kind of submit button
    game: Game,
    curr_board: StringField,
    curr_hand: StringField,
    curr_move: StringField,
    submit: Button,
}

enum InputMode {
    Normal,
    Editing,
}

enum State {
    Setup,
    Gaming,
}

enum ActiveBox {
    NumPlayers,
    Language,
    Start,
}

/// A new-type representing a string field with a label.
#[derive(Debug)]

struct StringField {
    // whether this field is currently in focus
    selected: bool,
    // whether this field is at all editable (TODO: actually implement this)
    editable: bool,
    /// Label above the field
    label: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Text currently in the box
    input: String,
}

impl GameTurn {
    fn new(game: Game) -> Self {
        GameTurn {
            game,
            curr_board: StringField::new("Current Board".to_owned()),
            curr_hand: StringField::new("Current Player's hand".to_owned()),
            curr_move: StringField::new("Move".to_owned()),
            submit: Button::new("Submit Move".to_owned()),
        }
    }
}

impl StringField {
    fn new(label: String) -> Self {
        StringField {
            selected: false,
            editable: false,
            label,
            character_index: 0,
            input: String::new(),
        }
    }

    fn make_editable(&mut self) {
        self.editable = true
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

    fn submit_message(&mut self) {
        // somehow save this particular setting
        self.input.clear();
        self.reset_cursor();
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
            active_box: ActiveBox::NumPlayers,
        }
    }

    fn get_active_box(&mut self) -> Option<&mut StringField> {
        match self.active_box {
            ActiveBox::NumPlayers => Some(&mut self.num_players),
            ActiveBox::Language => Some(&mut self.language),
            ActiveBox::Start => None,
        }
    }

    fn select_next_box(&mut self) {
        match self.active_box {
            ActiveBox::NumPlayers => {
                self.active_box = ActiveBox::Language;
                self.num_players.selected = false;
                self.language.selected = true;
                self.start_button.selected = false;
            }
            ActiveBox::Language => {
                self.active_box = ActiveBox::Start;
                self.num_players.selected = false;
                self.language.selected = false;
                self.start_button.selected = true;
            }
            ActiveBox::Start => {
                self.active_box = ActiveBox::NumPlayers;
                self.num_players.selected = true;
                self.language.selected = false;
                self.start_button.selected = false;
            }
        }
    }

    fn on_key_press(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Tab => self.select_next_box(),
            KeyCode::Char(c) => self.get_active_box().unwrap().enter_char(c),

            KeyCode::Backspace => self.get_active_box().unwrap().delete_char(),
            KeyCode::Left => self.get_active_box().unwrap().move_cursor_left(),
            KeyCode::Right => self.get_active_box().unwrap().move_cursor_right(),
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame) {
        let [num_players_area, language_area, start_area] =
            Layout::vertical(Constraint::from_lengths([3, 3, 1])).areas(frame.area());

        frame.render_widget(&self.num_players, num_players_area);
        frame.render_widget(&self.language, language_area);

        frame.render_widget(&self.start_button, start_area);

        let (active_area, active_offset) = match self.active_box {
            ActiveBox::Language => (language_area, self.language.character_index),
            ActiveBox::NumPlayers => (num_players_area, self.num_players.character_index),
            ActiveBox::Start => (start_area, 0),
        };
        if active_area != start_area {
            let cursor_pos =
                Position::new(active_area.x + active_offset as u16 + 1, active_area.y + 1);
            frame.set_cursor_position(cursor_pos);
        }
    }
}

impl App {
    fn new() -> Self {
        Self {
            settings: Settings::new(),
            state: State::Setup,
            game_state: None,
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Some(key) = event::read()?.as_key_press_event() {
            self.on_key_press(key);
        }
        Ok(())
    }

    fn on_key_press(&mut self, event: KeyEvent) {
        match self.state {
            State::Setup => match event.code {
                KeyCode::Enter => match self.settings.active_box {
                    ActiveBox::Language | ActiveBox::NumPlayers => {
                        self.settings.get_active_box().unwrap().submit_message()
                    }
                    ActiveBox::Start => self.start_game(),
                },
                _ => self.settings.on_key_press(event),
            },
            State::Gaming => todo!(),
        }
    }

    fn start_game(&mut self) {
        let num_players = self
            .settings
            .num_players
            .input
            .parse::<u32>()
            .expect("invalid number of players");
        let language = Language::by_name(&self.settings.language.input).expect("invalid language");

        let mut players = Vec::new();
        for i in 0..num_players {
            let player = Player::new(format!("player {}", i));
            players.push(player);
        }

        let layout = BoardLayout::from_fn((15, 15), standard_board_layout);

        let game = Game::new(players, layout, language);
        self.game_state = Some(GameTurn::new(game));
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
    }

    fn render(&self, frame: &mut Frame) {
        match self.state {
            State::Setup => self.settings.render(frame),
            State::Gaming => todo!(),
        }
    }
}

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind, KeyModifiers,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::io;
use std::sync::{Arc, Mutex};
struct App {
    show_popup: bool,
}
#[derive(Default)]
struct ScrollState {
    pub vertical_scroll: usize,
}

impl App {
    fn new() -> App {
        App { show_popup: false }
    }
}
#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

struct Input {
    input: String,
    cursor_position: usize,
    input_mode: InputMode,
    messages: Vec<String>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            cursor_position: 0,
        }
    }
}

impl Input {
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    fn submit_message(&mut self) {
        self.messages.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }
}
fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let app = App::new();
    let input = Input::default();
    let scroll_state = Arc::new(Mutex::new(ScrollState::default()));
    let res = run_app(&mut terminal, app, input, scroll_state);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    println!("Bye from Hyprland Wiki!");

    if let Err(err) = res {
        println!("{err:?}");
    }
    Ok(())
}
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut input: Input,
    scroll: Arc<Mutex<ScrollState>>,
) -> io::Result<()> {
    loop {
        let terminal_size = terminal.size()?; // Get the terminal size
        terminal.draw(|f| ui(f, &app, &input, &mut scroll.lock().unwrap()))?;
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let ctrl_pressed = key.modifiers.contains(KeyModifiers::CONTROL);
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Up => {
                        let mut scroll = scroll.lock().unwrap(); // Lock the Mutex
                        if scroll.vertical_scroll > 0 {
                            scroll.vertical_scroll = scroll.vertical_scroll.saturating_sub(1);
                        }
                    }
                    KeyCode::Down => {
                        let mut scroll = scroll.lock().unwrap();
                        let max_scroll = (input.messages.len() as i32 - terminal_size.height as i32)
                            .max(0) as usize;
                        if scroll.vertical_scroll < max_scroll {
                            scroll.vertical_scroll += 1;
                        }
                    }

                    KeyCode::Char('f') if ctrl_pressed => {
                        app.show_popup = !app.show_popup;
                        match input.input_mode {
                            InputMode::Normal => {
                                input.input_mode = InputMode::Editing;
                            }
                            InputMode::Editing => {
                                input.input_mode = InputMode::Normal;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if input.input_mode == InputMode::Editing {
                            input.submit_message();
                        }
                    }
                    KeyCode::Char(to_insert) => {
                        if input.input_mode == InputMode::Editing {
                            input.enter_char(to_insert);
                        }
                    }
                    KeyCode::Backspace => {
                        if input.input_mode == InputMode::Editing {
                            input.delete_char();
                        }
                    }
                    KeyCode::Left => {
                        if input.input_mode == InputMode::Editing {
                            input.move_cursor_left();
                        }
                    }
                    KeyCode::Right => {
                        if input.input_mode == InputMode::Editing {
                            input.move_cursor_right();
                        }
                    }
                    _ => {}
                }
            }
        } else if let event::Event::Mouse(mouse_event) = event::read()? {
            // Handle mouse events
            match mouse_event.kind {
                MouseEventKind::ScrollDown => {
                    let mut scroll = scroll.lock().unwrap();
                    let max_scroll =
                        (input.messages.len() as i32 - terminal_size.height as i32).max(0) as usize;
                    if scroll.vertical_scroll < max_scroll {
                        scroll.vertical_scroll += 1;
                    }
                }
                MouseEventKind::ScrollUp => {
                    let mut scroll = scroll.lock().unwrap();
                    if scroll.vertical_scroll > 0 {
                        scroll.vertical_scroll -= 1;
                    }
                }
                _ => {}
            }
        }
    }
}
fn ui<B: Backend>(f: &mut Frame<B>, app: &App, input: &Input, scroll: &mut ScrollState) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(size);
    let block = Block::default().borders(Borders::ALL);
    f.render_widget(block, chunks[0]);
    let block = Block::default().borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
    let max_scroll = (input.messages.len() as i32 - chunks[1].height as i32).max(0) as usize;
    let clamped_scroll = scroll.vertical_scroll.min(max_scroll); // Ensure not exceeding max_scroll
    let visible_messages: Vec<ListItem> = input
        .messages
        .iter()
        .skip(clamped_scroll)
        .take(chunks[1].height as usize)
        .enumerate()
        .map(|(_, m)| {
            let content = Line::from(Span::raw(m));
            ListItem::new(content)
        })
        .collect();
    let message_list = List::new(visible_messages)
        .block(Block::default().borders(Borders::ALL))
        .start_corner(Corner::TopLeft);
    f.render_widget(message_list, chunks[1]);
    if app.show_popup {
        let area = centered_rect(40, 10, size);

        let input_field = Paragraph::new(input.input.as_str())
            .style(match input.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Green),
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Find")
                    .title_alignment(Alignment::Center),
            );
        match input.input_mode {
            InputMode::Normal => {}

            InputMode::Editing => {
                f.set_cursor(area.x + input.cursor_position as u16 + 1, area.y + 1)
            }
        }

        f.render_widget(Clear, area);
        f.render_widget(input_field, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 5),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

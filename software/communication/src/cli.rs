use std::{str::FromStr, time::Duration};

use communication::{HostCommand, driver::DeviceDriver};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use anyhow::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
};

type DeviceEvent = communication::DeviceEvent<String>;

#[tokio::main]
async fn main() -> Result<()> {
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal).await;
    ratatui::restore();
    app_result
}

enum Message {
    HostCommand(HostCommand),
    DeviceEvent(DeviceEvent),
    HostLog {
        level: communication::LogMessageLevel,
        message: String,
    },
}

struct App {
    history_cursor: usize,
    input_history: Vec<String>,
    input: String,
    character_index: usize,
    input_mode: InputMode,
    messages: Vec<Message>,
}

#[derive(Debug, Clone)]
enum InputMode {
    Monitor,
    Command,
}

enum AppEvent {
    Exit,
    SetInputMode(InputMode),
    SubmitCommand,
    DisplayMessage(Message),
    EnterChar(char),
    DeleteChar,
    MoveCursorLeft,
    MoveCursorRight,
    ScrollUpHistory,
    ScrollDownHistory,
}

impl App {
    fn new() -> Self {
        Self {
            history_cursor: 0,
            input_history: Vec::new(),
            input: String::new(),
            input_mode: InputMode::Monitor,
            messages: Vec::new(),
            character_index: 0,
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
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
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

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    async fn submit_message(&mut self, message: Message) {
        self.messages.push(message);
        self.input.clear();
        self.reset_cursor();
    }

    // async fn display_device_event(
    //     app_event_tx: &mpsc::Sender<AppEvent>,
    //     event: DeviceEvent,
    // ) -> Result<()> {
    //     match event {
    //         DeviceEvent::LogMessage { level: _, message } => {
    //             app_event_tx.try_send(AppEvent::DisplayMessage(message))
    //         }
    //         _ => app_event_tx.try_send(AppEvent::DisplayMessage(
    //             "unimplemented device event.".into(),
    //         )),
    //     }
    //     .map_err(|e| anyhow::anyhow!("Failed to send device event: {e}"))
    // }

    async fn serial_port_com(
        app_event_tx: mpsc::Sender<AppEvent>,
        mut host_cmd_rx: mpsc::Receiver<HostCommand>,
        quit_notify: CancellationToken,
    ) -> Result<()> {
        // TODO: Make port configurable

        let mut driver = DeviceDriver::new(std::path::Path::new("/dev/ttyUSB0"), 115_200)?;
        driver.reset()?;

        while !driver.connection_established::<String>() {
            if quit_notify.is_cancelled() {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }

        app_event_tx
            .try_send(AppEvent::DisplayMessage(Message::HostLog {
                level: communication::LogMessageLevel::Info,
                message: "Connection established.".into(),
            }))
            .ok();

        while !quit_notify.is_cancelled() {
            while let Ok(cmd) = host_cmd_rx.try_recv() {
                driver.send_command(cmd)?;
            }

            if let Some(event) = driver.recv_event::<String>()? {
                app_event_tx
                    .send(AppEvent::DisplayMessage(Message::DeviceEvent(event)))
                    .await?;
            } else {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }

        Ok(())
    }

    async fn process_key_events(
        mut input_mode: InputMode,
        app_event_tx: mpsc::Sender<AppEvent>,
        quit_notify: CancellationToken,
    ) -> Result<()> {
        while !quit_notify.is_cancelled() {
            let event = if let Ok(event) = tokio::task::spawn_blocking(event::read).await {
                event?
            } else {
                continue;
            };

            if let Event::Key(key) = event {
                match input_mode {
                    InputMode::Monitor => match key.code {
                        KeyCode::Char('s') => {
                            input_mode = InputMode::Command;
                            app_event_tx
                                .send(AppEvent::SetInputMode(InputMode::Command))
                                .await?;
                        }
                        KeyCode::Char('q') => {
                            app_event_tx.send(AppEvent::Exit).await?;
                            break;
                        }
                        _ => {}
                    },
                    InputMode::Command if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => app_event_tx.send(AppEvent::SubmitCommand).await?,
                        KeyCode::Char(to_insert) => {
                            app_event_tx.send(AppEvent::EnterChar(to_insert)).await?
                        }
                        KeyCode::Backspace => app_event_tx.send(AppEvent::DeleteChar).await?,
                        KeyCode::Left => app_event_tx.send(AppEvent::MoveCursorLeft).await?,
                        KeyCode::Right => app_event_tx.send(AppEvent::MoveCursorRight).await?,
                        KeyCode::Up => app_event_tx.send(AppEvent::ScrollUpHistory).await?,
                        KeyCode::Down => app_event_tx.send(AppEvent::ScrollDownHistory).await?,
                        KeyCode::Esc => {
                            app_event_tx
                                .send(AppEvent::SetInputMode(InputMode::Monitor))
                                .await?;
                            input_mode = InputMode::Monitor;
                        }
                        _ => {}
                    },
                    InputMode::Command => {}
                }
            }
        }

        Ok(())
    }

    async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let quit_notify = CancellationToken::new();
        let (app_event_tx, mut app_event_rx) = mpsc::channel::<AppEvent>(100);
        let (host_cmd_tx, host_cmd_rx) = mpsc::channel::<HostCommand>(10);

        let key_event_handle = {
            let app_event_tx = app_event_tx.clone();
            let input_mode = self.input_mode.clone();
            let quit_notify = quit_notify.clone();
            tokio::spawn(Self::process_key_events(
                input_mode,
                app_event_tx,
                quit_notify,
            ))
        };

        let serial_port_handle = {
            let app_event_tx = app_event_tx.clone();
            let quit_notify = quit_notify.clone();
            tokio::spawn(Self::serial_port_com(
                app_event_tx,
                host_cmd_rx,
                quit_notify,
            ))
        };

        terminal.draw(|frame| self.draw(frame))?;
        loop {
            let event = if let Some(event) = app_event_rx.recv().await {
                event
            } else {
                quit_notify.cancel();
                break;
            };

            match event {
                AppEvent::Exit => {
                    quit_notify.cancel();
                    break;
                }
                AppEvent::SetInputMode(mode) => {
                    self.input_mode = mode;
                }
                AppEvent::SubmitCommand => {
                    let input = self.input.clone();
                    let packet = HostCommand::from_str(input.as_str());

                    if let Err(e) = packet {
                        self.messages.push(Message::HostLog {
                            level: communication::LogMessageLevel::Error,
                            message: format!("Failed to parse command: {e}"),
                        });
                    } else {
                        let packet = packet.unwrap();
                        host_cmd_tx.send(packet.clone()).await?;
                        self.input_history.push(input.clone());
                        self.history_cursor = self.input_history.len();
                        self.submit_message(Message::HostCommand(packet)).await;
                    }
                }
                AppEvent::DisplayMessage(msg) => {
                    self.messages.push(msg);
                }
                AppEvent::EnterChar(c) => {
                    self.enter_char(c);
                }
                AppEvent::DeleteChar => {
                    self.delete_char();
                }
                AppEvent::MoveCursorLeft => {
                    self.move_cursor_left();
                }
                AppEvent::MoveCursorRight => {
                    self.move_cursor_right();
                }
                AppEvent::ScrollUpHistory => {
                    if self.history_cursor == 0 {
                        continue;
                    }

                    self.history_cursor -= 1;
                    self.input = self.input_history[self.history_cursor].clone();
                }
                AppEvent::ScrollDownHistory => {
                    self.history_cursor = (self.history_cursor + 1).min(self.input_history.len());

                    if self.history_cursor == self.input_history.len() {
                        self.input.clear();
                    } else {
                        self.input = self.input_history[self.history_cursor].clone();
                    }
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        key_event_handle.await??;
        serial_port_handle.await??;

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, messages_area] = vertical.areas(frame.area());

        let (msg, style) = match self.input_mode {
            InputMode::Monitor => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "s".bold(),
                    " to send commands.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Command => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " exit command mode, ".into(),
                    "Enter".bold(),
                    " dispatch the command".into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Monitor => Style::default(),
                InputMode::Command => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Command"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Monitor => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Command => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }

        let mut messages: Vec<ListItem> = Vec::new();

        let mut device_idx = 0;
        let mut host_idx = 0;

        for msg in self.messages.iter() {
            match msg {
                Message::HostCommand(cmd) => {
                    host_idx += 1;
                    let msg = vec![
                        " ⇨ ".bold().green(),
                        format!("{host_idx:04}").green(),
                        " ".into(),
                        cmd.to_string().bold(),
                    ];
                    messages.push(ListItem::new(Line::from(msg)));
                }
                Message::DeviceEvent(event) => {
                    device_idx += 1;
                    let mut msg = vec![
                        " ⇦ ".bold().magenta(),
                        format!("{device_idx:04}").magenta(),
                        " ".into(),
                    ];

                    match event {
                        DeviceEvent::LogMessage { level, message } => {
                            let prefix = match level {
                                communication::LogMessageLevel::Info => "[INFO]".blue(),
                                communication::LogMessageLevel::Warning => "[WARN]".yellow(),
                                communication::LogMessageLevel::Error => "[ERROR]".red(),
                            };
                            msg.push(prefix.into());
                            msg.push(" ".into());
                            msg.push(message.into());
                        }
                        _ => {
                            msg.push("<?>".into());
                        }
                    }

                    messages.push(ListItem::new(Line::from(msg)));
                }
                Message::HostLog { level, message } => {
                    let prefix = match level {
                        communication::LogMessageLevel::Info => "[INFO]".blue(),
                        communication::LogMessageLevel::Warning => "[WARN]".yellow(),
                        communication::LogMessageLevel::Error => "[ERROR]".red(),
                    };
                    let msg = vec![
                        " ⊙ ".bold().dim(),
                        "---- ".dim(),
                        prefix.into(),
                        " ".into(),
                        message.into(),
                    ];
                    messages.push(ListItem::new(Line::from(msg)));
                }
            }
        }

        messages.reverse();

        let messages = List::new(messages).block(Block::bordered().title("Events"));
        frame.render_widget(messages, messages_area);
    }
}

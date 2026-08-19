use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io};
use crate::app::{App, ActiveBlock};
use crate::ui;
use tui_input::backend::crossterm::EventHandler;

pub fn run_tui(app: &mut App) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>>
where
    <B as Backend>::Error: std::error::Error + 'static,
{
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        let event = event::read()?;
        
        match app.active_block {
            ActiveBlock::List => {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Char('n') => app.open_new_form(),
                        KeyCode::Char('e') => app.open_edit_form(),
                        KeyCode::Char('d') => {
                            if app.list_state.selected().is_some() {
                                app.active_block = ActiveBlock::DeleteConfirm;
                            }
                        },
                        KeyCode::Char('f') => app.toggle_favorite(),
                        KeyCode::Char('s') => app.active_block = ActiveBlock::Settings,
                        KeyCode::Char('/') => app.active_block = ActiveBlock::Search,
                        KeyCode::Enter => {
                            if let Some(i) = app.list_state.selected() {
                                if let Some(&idx) = app.filtered_commands.get(i) {
                                    let cmd = &app.config.commands[idx];
                                    app.command_to_run = Some(cmd.name.clone());
                                    return Ok(());
                                }
                            }
                        }
                        _ => {}
                    }
                } else if let Event::Mouse(mouse_event) = event {
                    match mouse_event.kind {
                        event::MouseEventKind::ScrollUp => app.previous(),
                        event::MouseEventKind::ScrollDown => app.next(),
                        event::MouseEventKind::Down(event::MouseButton::Left) => {
                            let rect = terminal.size().unwrap_or_default();
                            if mouse_event.row >= rect.height.saturating_sub(3) {
                                // Clicked in footer
                                let col = mouse_event.column;
                                if col >= 3 && col <= 18 {
                                    if let Some(i) = app.list_state.selected() {
                                        if let Some(&idx) = app.filtered_commands.get(i) {
                                            let cmd = &app.config.commands[idx];
                                            app.command_to_run = Some(cmd.name.clone());
                                            return Ok(());
                                        }
                                    }
                                } else if col >= 34 && col <= 48 {
                                    app.active_block = ActiveBlock::Search;
                                } else if col >= 49 && col <= 60 {
                                    app.open_new_form();
                                } else if col >= 61 && col <= 73 {
                                    app.open_edit_form();
                                } else if col >= 74 && col <= 88 {
                                    if app.list_state.selected().is_some() {
                                        app.active_block = ActiveBlock::DeleteConfirm;
                                    }
                                } else if col >= 89 && col <= 100 {
                                    app.toggle_favorite();
                                } else if col >= 101 && col <= 110 {
                                    app.should_quit = true;
                                }
                            } else {
                                // Calculate title height dynamically based on window height
                                let show_big_title = rect.height > 25;
                                let title_height = if show_big_title { 8 } else { 0 };
                                
                                if mouse_event.row >= title_height + 3 {
                                    // Clicked in list
                                    let clicked_row = (mouse_event.row - title_height - 3) as usize;
                                    let new_index = app.list_state.offset().saturating_add(clicked_row);
                                    if new_index < app.filtered_commands.len() {
                                        app.list_state.select(Some(new_index));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            },
            ActiveBlock::Search => {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter => {
                            app.active_block = ActiveBlock::List;
                        },
                        _ => {
                            app.search_input.handle_event(&Event::Key(key));
                            app.update_filter();
                        }
                    }
                }
            },
            ActiveBlock::Form => {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Esc => app.active_block = ActiveBlock::List,
                        KeyCode::Enter => app.submit_form(),
                        KeyCode::Tab | KeyCode::Down => {
                            app.form.active_field = (app.form.active_field + 1) % 7;
                        }
                        KeyCode::BackTab | KeyCode::Up => {
                            app.form.active_field = if app.form.active_field == 0 { 6 } else { app.form.active_field - 1 };
                        }
                        KeyCode::Char(' ') if app.form.active_field >= 5 => {
                            if app.form.active_field == 5 {
                                app.form.favorite = !app.form.favorite;
                            } else {
                                app.form.confirm = !app.form.confirm;
                            }
                        }
                        _ => {
                            match app.form.active_field {
                                0 => { app.form.name.handle_event(&Event::Key(key)); },
                                1 => { app.form.display_name.handle_event(&Event::Key(key)); },
                                2 => { app.form.description.handle_event(&Event::Key(key)); },
                                3 => { app.form.category.handle_event(&Event::Key(key)); },
                                4 => { app.form.command.handle_event(&Event::Key(key)); },
                                _ => {},
                            };
                        }
                    }
                } else if let Event::Mouse(mouse_event) = event {
                    if mouse_event.kind == event::MouseEventKind::Down(event::MouseButton::Left) {
                        let rect = terminal.size().unwrap_or_default();
                        let area = ui::centered_rect(60, 80, rect.into());
                        let start_y = area.y + 1; // margin(1)
                        let row = mouse_event.row;
                        let col = mouse_event.column;
                        
                        // Check if click is inside modal bounds
                        if col >= area.x + 1 && col <= area.x + area.width - 1 {
                            if row >= start_y && row < start_y + 3 {
                                app.form.active_field = 0;
                            } else if row >= start_y + 3 && row < start_y + 6 {
                                app.form.active_field = 1;
                            } else if row >= start_y + 6 && row < start_y + 9 {
                                app.form.active_field = 2;
                            } else if row >= start_y + 9 && row < start_y + 12 {
                                app.form.active_field = 3;
                            } else if row >= start_y + 12 && row < start_y + 15 {
                                app.form.active_field = 4;
                            } else if row == start_y + 15 {
                                app.form.favorite = !app.form.favorite;
                                app.form.active_field = 5;
                            } else if row == start_y + 16 {
                                app.form.confirm = !app.form.confirm;
                                app.form.active_field = 6;
                            }
                        }
                    }
                }
            },
            ActiveBlock::DeleteConfirm => {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => app.delete_selected(),
                        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => app.active_block = ActiveBlock::List,
                        _ => {}
                    }
                }
            },

            ActiveBlock::Settings => {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('s') => app.active_block = ActiveBlock::List,
                        KeyCode::Enter => {
                            if app.settings_state.selected() == Some(0) {
                                app.active_block = ActiveBlock::ThemeSelector;
                            }
                        },
                        _ => {}
                    }
                }
            },
            ActiveBlock::ThemeSelector => {
                if let Event::Key(key) = event {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => app.active_block = ActiveBlock::Settings,
                        KeyCode::Down | KeyCode::Char('j') => {
                            let max = crate::theme::get_all_themes().len();
                            let i = match app.theme_state.selected() {
                                Some(i) => if i >= max - 1 { 0 } else { i + 1 },
                                None => 0,
                            };
                            app.theme_state.select(Some(i));
                        },
                        KeyCode::Up | KeyCode::Char('k') => {
                            let max = crate::theme::get_all_themes().len();
                            let i = match app.theme_state.selected() {
                                Some(i) => if i == 0 { max - 1 } else { i - 1 },
                                None => 0,
                            };
                            app.theme_state.select(Some(i));
                        },
                        KeyCode::Enter => {
                            if let Some(i) = app.theme_state.selected() {
                                let themes = crate::theme::get_all_themes();
                                if let Some(theme) = themes.get(i) {
                                    app.config.preferences.theme = theme.name.to_string();
                                    app.save_config();
                                    app.active_block = ActiveBlock::Settings;
                                }
                            }
                        },
                        _ => {}
                    }
                }
            },
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

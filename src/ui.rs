use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::{App, ActiveBlock};

pub fn render(f: &mut Frame, app: &mut App) {
    let show_big_title = f.area().height > 25;
    let title_height = if show_big_title { 8 } else { 0 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_height), // ASCII Title
            Constraint::Length(3),            // Search bar
            Constraint::Min(3),               // Main content
            Constraint::Length(3),            // Footer
        ].as_ref())
        .split(f.area());

    if show_big_title {
        f.render_widget(render_ascii_title(), chunks[0]);
    }

    // Search Bar
    let search_title = Line::from(vec![
        Span::styled(" Search (/) ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]);
    let search_style = if app.active_block == ActiveBlock::Search {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    
    let search_text = format!(" {} ", app.search_input.value());
    let search_widget = Paragraph::new(search_text)
        .block(Block::default().borders(Borders::ALL).title(search_title).border_style(search_style));
    f.render_widget(search_widget, chunks[1]);
    
    if app.active_block == ActiveBlock::Search {
        f.set_cursor_position((
            chunks[1].x + 2 + app.search_input.visual_cursor() as u16,
            chunks[1].y + 1,
        ));
    }

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if f.area().width > 60 {
            vec![Constraint::Percentage(40), Constraint::Percentage(60)]
        } else {
            vec![Constraint::Percentage(100)] // responsive single pane
        })
        .split(chunks[2]);

    if app.config.commands.is_empty() {
        let empty_msg = Paragraph::new("No commands yet!\n\nPress 'N' to create your first command.")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("CmdDeck"));
        f.render_widget(empty_msg, chunks[2]);
    } else {
        // List
        let items: Vec<ListItem> = app.filtered_commands.iter().map(|&idx| {
            let cmd = &app.config.commands[idx];
            let prefix = if cmd.favorite { "★ " } else { "  " };
            let title = cmd.display_name.as_deref().unwrap_or(&cmd.name);
            ListItem::new(format!("{}{}", prefix, title))
        }).collect();

        let list_style = if app.active_block == ActiveBlock::List {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Commands").border_style(list_style))
            .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD));
        f.render_stateful_widget(list, main_chunks[0], &mut app.list_state);

        // Details Pane
        if main_chunks.len() > 1 {
            if let Some(i) = app.list_state.selected() {
                if let Some(&idx) = app.filtered_commands.get(i) {
                    let cmd = &app.config.commands[idx];
                    let title = cmd.display_name.as_deref().unwrap_or(&cmd.name);
                    let mut details_text = vec![
                        Line::from(vec![Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(&cmd.name)]),
                        Line::from(vec![Span::styled("Display: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(title)]),
                        Line::from(vec![Span::styled("Category: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(cmd.category.as_deref().unwrap_or("None"))]),
                        Line::from(vec![Span::styled("Favorite: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(if cmd.favorite { "Yes" } else { "No" })]),
                        Line::from(vec![Span::styled("Confirm: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(if cmd.confirmation_required { "Yes" } else { "No" })]),
                        Line::raw(""),
                        Line::from(Span::styled("Command:", Style::default().add_modifier(Modifier::BOLD))),
                        Line::raw(&cmd.command),
                    ];
                    
                    if let Some(desc) = &cmd.description {
                        details_text.insert(5, Line::from(vec![Span::styled("Description: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(desc)]));
                    }
                    
                    let details = Paragraph::new(details_text)
                        .block(Block::default().borders(Borders::ALL).title("Details"))
                        .wrap(Wrap { trim: true });
                    f.render_widget(details, main_chunks[1]);
                }
            } else {
                let empty = Paragraph::new("No match.").block(Block::default().borders(Borders::ALL).title("Details"));
                f.render_widget(empty, main_chunks[1]);
            }
        }
    }

    // Footer
    let footer_content = if app.active_block == ActiveBlock::Search {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(" Esc ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel  |  "),
            Span::styled(" Enter ", Style::default().bg(Color::Green).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Select  "),
        ])
    } else {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(" Enter ", Style::default().bg(Color::Green).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Run  |  "),
            Span::styled(" ↑/↓ ", Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Move  |  "),
            Span::styled(" / ", Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Search  |  "),
            Span::styled(" N ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw(" New  |  "),
            Span::styled(" E ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw(" Edit  |  "),
            Span::styled(" D ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Delete  |  "),
            Span::styled(" F ", Style::default().bg(Color::Magenta).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Fav  |  "),
            Span::styled(" Q ", Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit  "),
        ])
    };
    
    let footer = Paragraph::new(footer_content)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);
    f.render_widget(footer, chunks[3]);

    // Modals
    if app.active_block == ActiveBlock::Form {
        draw_form_modal(f, app);
    } else if app.active_block == ActiveBlock::DeleteConfirm {
        draw_delete_modal(f);
    }
}

fn render_ascii_title<'a>() -> Paragraph<'a> {
    let lines = vec![
        Line::raw(""),
        Line::from(Span::styled("                    ██ ████████▄ █████████ ▄███████ ██    ██ ", Style::default().fg(Color::Rgb(100, 200, 255)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(" ▄████▄  ▄███▄███▄  ▄████▄██ ██    ▀██ ██▀▀▀▀▀▀▀ ██▀▀▀▀▀▀ ██  ▄██▀ ", Style::default().fg(Color::Rgb(100, 150, 255)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("██▀  ▀▀  ██▀ ██▀ ██ ██▀  ▀██ ██     ██ ███████   ██       █████▀   ", Style::default().fg(Color::Rgb(150, 100, 255)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("██▄  ▄▄  ██  ██  ██ ██▄  ▄██ ██    ▄██ ██▄▄▄▄▄▄▄ ██▄▄▄▄▄▄ ██  ▀██▄ ", Style::default().fg(Color::Rgb(255, 100, 200)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(" ▀████▀  ██  ██  ██ ▀████▀██ ████████▀ █████████ ▀███████ ██    ██ ", Style::default().fg(Color::Rgb(255, 100, 150)).add_modifier(Modifier::BOLD))),
    ];
    Paragraph::new(lines).alignment(Alignment::Center)
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
}

fn draw_form_modal(f: &mut Frame, app: &mut App) {
    let area = centered_rect(60, 80, f.area());
    f.render_widget(Clear, area);
    
    let block = Block::default().title(if app.form.is_edit { "Edit Command" } else { "New Command" }).borders(Borders::ALL);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Display
            Constraint::Length(3), // Description
            Constraint::Length(3), // Category
            Constraint::Length(3), // Command
            Constraint::Length(1), // Favorite
            Constraint::Length(1), // Confirm
            Constraint::Min(2),    // Footer
        ].as_ref())
        .split(area);
        
    let fields = [
        ("Name (alias) *required*", &app.form.name),
        ("Display Name", &app.form.display_name),
        ("Description", &app.form.description),
        ("Category", &app.form.category),
        ("Command String *required*", &app.form.command),
    ];
    
    for (i, (title, input)) in fields.iter().enumerate() {
        let style = if app.form.active_field == i { Style::default().fg(Color::Yellow) } else { Style::default() };
        let p = Paragraph::new(input.value()).block(Block::default().borders(Borders::ALL).title(*title).border_style(style));
        f.render_widget(p, chunks[i]);
    }
    
    let fav_style = if app.form.active_field == 5 { Style::default().fg(Color::Yellow) } else { Style::default() };
    let conf_style = if app.form.active_field == 6 { Style::default().fg(Color::Yellow) } else { Style::default() };
    f.render_widget(Paragraph::new(format!("[{}] Favorite", if app.form.favorite { "x" } else { " " })).style(fav_style), chunks[5]);
    f.render_widget(Paragraph::new(format!("[{}] Require Confirmation", if app.form.confirm { "x" } else { " " })).style(conf_style), chunks[6]);
    
    let help = Paragraph::new("Tab: Next Field | Enter: Save | Esc: Cancel").alignment(Alignment::Center).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[7]);
    
    if app.form.active_field < 5 {
        let active_input = match app.form.active_field {
            0 => &app.form.name,
            1 => &app.form.display_name,
            2 => &app.form.description,
            3 => &app.form.category,
            4 => &app.form.command,
            _ => unreachable!(),
        };
        f.set_cursor_position((
            chunks[app.form.active_field].x + 1 + active_input.visual_cursor() as u16,
            chunks[app.form.active_field].y + 1,
        ));
    }
}

fn draw_delete_modal(f: &mut Frame) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);
    let p = Paragraph::new("Are you sure you want to delete this command?\n\nPress 'y' to confirm, or 'Esc' to cancel.")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Delete Command").border_style(Style::default().fg(Color::Red)));
    f.render_widget(p, area);
}

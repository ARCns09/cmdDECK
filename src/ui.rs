use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{App, ActiveBlock};
use crate::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App) {
    let theme = app.get_theme();
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
        f.render_widget(render_ascii_title(&theme), chunks[0]);
    }

    // Search Bar
    let search_title = Line::from(vec![
        Span::styled(" Search (/) ", Style::default().fg(theme.btn_search).add_modifier(Modifier::BOLD)),
    ]);
    let search_style = if app.active_block == ActiveBlock::Search {
        Style::default().fg(theme.border_active)
    } else {
        Style::default().fg(theme.border)
    };
    
    let search_text = format!(" {} ", app.search_input.value());
    let search_widget = Paragraph::new(search_text)
        .style(Style::default().fg(theme.text_primary))
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
            .style(Style::default().fg(theme.text_secondary))
            .block(Block::default().borders(Borders::ALL).title("CmdDeck").border_style(Style::default().fg(theme.border)));
        f.render_widget(empty_msg, chunks[2]);
    } else {
        // List
        let items: Vec<ListItem> = app.filtered_commands.iter().map(|&idx| {
            let cmd = &app.config.commands[idx];
            let fav = if cmd.favorite { "★ " } else { "  " };
            let title = format!("{}{}", fav, cmd.name);
            ListItem::new(title).style(Style::default().fg(theme.text_primary))
        }).collect();

        let list_style = if app.active_block == ActiveBlock::List {
            Style::default().fg(theme.border_active)
        } else {
            Style::default().fg(theme.border)
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Commands").border_style(list_style))
            .highlight_style(Style::default().bg(theme.list_selected_bg).fg(theme.list_selected_fg).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
        
        f.render_stateful_widget(list, main_chunks[0], &mut app.list_state);

        // Details
        if let Some(i) = app.list_state.selected() {
            if let Some(&idx) = app.filtered_commands.get(i) {
                let cmd = &app.config.commands[idx];
                let mut text = vec![
                    Line::from(vec![Span::styled("Name: ", Style::default().fg(theme.text_secondary)), Span::styled(&cmd.name, Style::default().fg(theme.text_primary))]),
                    Line::from(""),
                ];
                
                if let Some(disp) = &cmd.display_name {
                    text.push(Line::from(vec![Span::styled("Display Name: ", Style::default().fg(theme.text_secondary)), Span::raw(disp)]));
                    text.push(Line::from(""));
                }
                
                if let Some(desc) = &cmd.description {
                    text.push(Line::from(vec![Span::styled("Description: ", Style::default().fg(theme.text_secondary)), Span::raw(desc)]));
                    text.push(Line::from(""));
                }
                
                text.push(Line::from(vec![Span::styled("Command String:", Style::default().fg(theme.text_secondary))]));
                text.push(Line::from(vec![Span::styled(format!("$ {}", cmd.command), Style::default().fg(theme.btn_run).add_modifier(Modifier::BOLD))]));
                
                let details = Paragraph::new(text)
                    .style(Style::default().fg(theme.text_primary))
                    .block(Block::default().borders(Borders::ALL).title("Details").border_style(Style::default().fg(theme.border)));
                if main_chunks.len() > 1 {
                    f.render_widget(details, main_chunks[1]);
                }
            }
        }
    }

    // Footer
    let footer_content = if app.active_block == ActiveBlock::Search {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(" Esc ", Style::default().bg(theme.btn_delete).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel  |  "),
            Span::styled(" Enter ", Style::default().bg(theme.btn_run).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Select  "),
        ])
    } else {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(" Enter ", Style::default().bg(theme.btn_run).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Run  |  "),
            Span::styled(" ↑/↓ ", Style::default().bg(theme.btn_move).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Move  |  "),
            Span::styled(" / ", Style::default().bg(theme.btn_search).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Search  |  "),
            Span::styled(" N ", Style::default().bg(theme.btn_new).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw(" New  |  "),
            Span::styled(" E ", Style::default().bg(theme.btn_edit).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw(" Edit  |  "),
            Span::styled(" D ", Style::default().bg(theme.btn_delete).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Delete  |  "),
            Span::styled(" F ", Style::default().bg(theme.btn_fav).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Fav  |  "),
            Span::styled(" S ", Style::default().bg(theme.btn_settings).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Settings  |  "),
            Span::styled(" Q ", Style::default().bg(theme.btn_quit).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit  "),
        ])
    };
    
    let footer = Paragraph::new(footer_content)
        .style(Style::default().fg(theme.text_primary))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)))
        .alignment(Alignment::Left);
    f.render_widget(footer, chunks[3]);

    // Modals
    if app.active_block == ActiveBlock::Form {
        draw_form_modal(f, app, &theme);
    } else if app.active_block == ActiveBlock::DeleteConfirm {
        draw_delete_modal(f, &theme);
    } else if app.active_block == ActiveBlock::Settings {
        draw_settings_modal(f, app, &theme);
    } else if app.active_block == ActiveBlock::ThemeSelector {
        draw_theme_selector_modal(f, app, &theme);
    }
}

fn render_ascii_title<'a>(theme: &Theme) -> Paragraph<'a> {
    let lines = vec![
        Line::raw(""),
        Line::from(Span::styled("                      ______  ______________ __  ", Style::default().fg(theme.title_gradient[0]).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  _________ ___  ____/ / __ \\/ ____/ ____/ //_/  ", Style::default().fg(theme.title_gradient[1]).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(" / ___/ __ `__ \\/ __  / / / / __/ / /   / ,<     ", Style::default().fg(theme.title_gradient[2]).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("/ /__/ / / / / / /_/ / /_/ / /___/ /___/ /| |    ", Style::default().fg(theme.title_gradient[3]).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("\\___/_/ /_/ /_/\\__,_/_____/_____/\\____/_/ |_|    ", Style::default().fg(theme.title_gradient[4]).add_modifier(Modifier::BOLD))),
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

fn draw_form_modal(f: &mut Frame, app: &mut App, theme: &Theme) {
    let area = centered_rect(60, 80, f.area());
    f.render_widget(Clear, area);
    
    let block = Block::default().title(if app.form.is_edit { "Edit Command" } else { "New Command" }).borders(Borders::ALL).border_style(Style::default().fg(theme.border));
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
        let style = if app.form.active_field == i { Style::default().fg(theme.border_active) } else { Style::default().fg(theme.border) };
        let p = Paragraph::new(input.value()).style(Style::default().fg(theme.text_primary)).block(Block::default().borders(Borders::ALL).title(*title).border_style(style));
        f.render_widget(p, chunks[i]);
    }
    
    let fav_style = if app.form.active_field == 5 { Style::default().fg(theme.border_active) } else { Style::default().fg(theme.text_primary) };
    let conf_style = if app.form.active_field == 6 { Style::default().fg(theme.border_active) } else { Style::default().fg(theme.text_primary) };
    f.render_widget(Paragraph::new(format!("[{}] Favorite", if app.form.favorite { "x" } else { " " })).style(fav_style), chunks[5]);
    f.render_widget(Paragraph::new(format!("[{}] Require Confirmation", if app.form.confirm { "x" } else { " " })).style(conf_style), chunks[6]);
    
    let help = Paragraph::new("Tab: Next Field | Enter: Save | Esc: Cancel").alignment(Alignment::Center).style(Style::default().fg(theme.text_secondary));
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

fn draw_delete_modal(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);
    let p = Paragraph::new("Are you sure you want to delete this command?\n\nPress 'y' to confirm, or 'Esc' to cancel.")
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_primary))
        .block(Block::default().borders(Borders::ALL).title("Delete Command").border_style(Style::default().fg(theme.btn_delete)));
    f.render_widget(p, area);
}

fn draw_settings_modal(f: &mut Frame, app: &mut App, theme: &Theme) {
    let area = centered_rect(40, 30, f.area());
    f.render_widget(Clear, area);

    let items = vec![
        ListItem::new("🎨 Themes").style(Style::default().fg(theme.text_primary)),
        ListItem::new("🔄 Check for Updates").style(Style::default().fg(theme.text_secondary)),
        ListItem::new("ℹ️ About CmdDeck").style(Style::default().fg(theme.text_secondary)),
    ];

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Settings").border_style(Style::default().fg(theme.border_active)))
        .highlight_style(Style::default().bg(theme.list_selected_bg).fg(theme.list_selected_fg).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.settings_state);
}

fn draw_theme_selector_modal(f: &mut Frame, app: &mut App, theme: &Theme) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);

    let available_themes = crate::theme::get_all_themes();
    
    let items: Vec<ListItem> = available_themes.iter().map(|t| {
        let prefix = if t.name == app.config.preferences.theme { "✓ " } else { "  " };
        ListItem::new(format!("{}{}", prefix, t.name)).style(Style::default().fg(theme.text_primary))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Select Theme").border_style(Style::default().fg(theme.border_active)))
        .highlight_style(Style::default().bg(theme.list_selected_bg).fg(theme.list_selected_fg).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.theme_state);
}

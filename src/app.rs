use ratatui::widgets::ListState;
use tui_input::Input;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::models::CmdEntry;
use crate::config::Config;
use crate::theme::{self, Theme};

#[derive(PartialEq)]
pub enum ActiveBlock {
    List,
    Search,
    Form,
    DeleteConfirm,
    Settings,
    ThemeSelector,
}

pub struct App {
    pub config: Config,
    pub list_state: ListState,
    pub should_quit: bool,
    pub command_to_run: Option<String>,
    
    pub active_block: ActiveBlock,
    pub search_input: Input,
    pub filtered_commands: Vec<usize>,

    pub form: CommandForm,
    pub settings_state: ListState,
    pub theme_state: ListState,
}

pub struct CommandForm {
    pub is_edit: bool,
    pub edit_idx: Option<usize>,
    pub active_field: usize,
    pub name: Input,
    pub display_name: Input,
    pub description: Input,
    pub category: Input,
    pub command: Input,
    pub favorite: bool,
    pub confirm: bool,
}

impl Default for CommandForm {
    fn default() -> Self {
        Self {
            is_edit: false,
            edit_idx: None,
            active_field: 0,
            name: Input::default(),
            display_name: Input::default(),
            description: Input::default(),
            category: Input::default(),
            command: Input::default(),
            favorite: false,
            confirm: false,
        }
    }
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut settings_state = ListState::default();
        settings_state.select(Some(0)); // Pre-select "Themes" option
        
        let mut theme_state = ListState::default();
        theme_state.select(Some(0)); // Will be updated when opened

        let mut app = Self {
            config,
            list_state: ListState::default(),
            should_quit: false,
            command_to_run: None,
            active_block: ActiveBlock::List,
            search_input: Input::default(),
            filtered_commands: Vec::new(),
            form: CommandForm::default(),
            settings_state,
            theme_state,
        };
        app.update_filter();
        app
    }

    pub fn get_theme(&self) -> Theme {
        theme::get_theme_by_name(&self.config.preferences.theme)
    }

    pub fn update_filter(&mut self) {
        let matcher = SkimMatcherV2::default();
        let query = self.search_input.value();
        
        let mut indices: Vec<(usize, i64)> = self.config.commands
            .iter()
            .enumerate()
            .filter_map(|(i, cmd)| {
                if query.is_empty() {
                    let score = if cmd.favorite { 1000 } else { 0 };
                    Some((i, score))
                } else {
                    let text = format!("{} {} {} {}", 
                        cmd.name, 
                        cmd.display_name.as_deref().unwrap_or(""), 
                        cmd.description.as_deref().unwrap_or(""), 
                        cmd.category.as_deref().unwrap_or("")
                    );
                    if let Some(score) = matcher.fuzzy_match(&text, query) {
                        Some((i, score + if cmd.favorite { 50 } else { 0 }))
                    } else {
                        None
                    }
                }
            })
            .collect();
            
        indices.sort_by(|a, b| b.1.cmp(&a.1));
        self.filtered_commands = indices.into_iter().map(|(i, _)| i).collect();
        
        if self.filtered_commands.is_empty() {
            self.list_state.select(None);
        } else {
            if self.list_state.selected().is_none() || self.list_state.selected().unwrap() >= self.filtered_commands.len() {
                self.list_state.select(Some(0));
            }
        }
    }

    pub fn next(&mut self) {
        if self.filtered_commands.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => if i >= self.filtered_commands.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered_commands.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => if i == 0 { self.filtered_commands.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
    }
    
    pub fn save_config(&self) {
        let _ = self.config.save();
    }
    
    pub fn open_new_form(&mut self) {
        self.form = CommandForm::default();
        self.active_block = ActiveBlock::Form;
    }
    
    pub fn open_edit_form(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(&idx) = self.filtered_commands.get(i) {
                let cmd = &self.config.commands[idx];
                self.form = CommandForm {
                    is_edit: true,
                    edit_idx: Some(idx),
                    active_field: 0,
                    name: Input::default().with_value(cmd.name.clone()),
                    display_name: Input::default().with_value(cmd.display_name.clone().unwrap_or_default()),
                    description: Input::default().with_value(cmd.description.clone().unwrap_or_default()),
                    category: Input::default().with_value(cmd.category.clone().unwrap_or_default()),
                    command: Input::default().with_value(cmd.command.clone()),
                    favorite: cmd.favorite,
                    confirm: cmd.confirmation_required,
                };
                self.active_block = ActiveBlock::Form;
            }
        }
    }
    
    pub fn submit_form(&mut self) {
        if self.form.name.value().trim().is_empty() || self.form.command.value().trim().is_empty() {
            return; // invalid
        }
        
        let new_cmd = CmdEntry {
            name: self.form.name.value().trim().to_string(),
            display_name: if self.form.display_name.value().trim().is_empty() { None } else { Some(self.form.display_name.value().trim().to_string()) },
            description: if self.form.description.value().trim().is_empty() { None } else { Some(self.form.description.value().trim().to_string()) },
            category: if self.form.category.value().trim().is_empty() { None } else { Some(self.form.category.value().trim().to_string()) },
            command: self.form.command.value().trim().to_string(),
            favorite: self.form.favorite,
            confirmation_required: self.form.confirm,
            working_directory: None,
        };
        
        if self.form.is_edit {
            if let Some(idx) = self.form.edit_idx {
                self.config.commands[idx] = new_cmd;
            }
        } else {
            if self.config.commands.iter().any(|c| c.name == new_cmd.name) {
                return; // invalid, name must be unique
            }
            self.config.commands.push(new_cmd);
        }
        
        self.save_config();
        self.update_filter();
        self.active_block = ActiveBlock::List;
    }
    
    pub fn delete_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(&idx) = self.filtered_commands.get(i) {
                self.config.commands.remove(idx);
                self.save_config();
                self.update_filter();
            }
        }
        self.active_block = ActiveBlock::List;
    }
    
    pub fn toggle_favorite(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(&idx) = self.filtered_commands.get(i) {
                self.config.commands[idx].favorite = !self.config.commands[idx].favorite;
                self.save_config();
                self.update_filter();
            }
        }
    }
}

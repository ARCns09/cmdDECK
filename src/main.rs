mod app;
mod config;
mod models;
mod tui;
mod ui;

use clap::{Parser, Subcommand};
use config::Config;
use std::process;
use std::env;
use app::App;

#[derive(Parser)]
#[command(name = "cm")]
#[command(about = "CmdDeck - A lightweight command manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Command alias to run directly
    alias: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new command
    Add,
    /// List all commands
    List,
    /// Edit an existing command
    Edit,
    /// Delete a command
    Delete,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = Config::load();

    if let Some(alias) = cli.alias {
        if let Err(e) = run_command_alias(&config, &alias) {
            eprintln!("{}", e);
            process::exit(1);
        }
        return Ok(());
    }

    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Add => println!("Add command (TUI modal coming soon)"),
            Commands::List => {
                for cmd in &config.commands {
                    println!("{} - {}", cmd.name, cmd.display_name.as_deref().unwrap_or(""));
                }
            }
            Commands::Edit => println!("Edit command (TUI modal coming soon)"),
            Commands::Delete => println!("Delete command (TUI modal coming soon)"),
        }
        return Ok(());
    }

    // Default: Open TUI
    let mut app = App::new(config);
    
    loop {
        tui::run_tui(&mut app)?;

        if let Some(alias) = app.command_to_run.take() {
            if let Err(e) = run_command_alias(&app.config, &alias) {
                eprintln!("\n{}", e);
                println!("Press Enter to return to CmdDeck...");
                let mut s = String::new();
                let _ = std::io::stdin().read_line(&mut s);
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn run_command_alias(config: &Config, alias: &str) -> Result<(), String> {
    let entry = config.commands.iter().find(|c| c.name == alias);
    
    match entry {
        Some(cmd) => {
            let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
            
            let mut process = process::Command::new(&shell);
            process.arg("-c").arg(&cmd.command);
            
            if let Some(ref dir) = cmd.working_directory {
                process.current_dir(dir);
            }
            
            if cmd.confirmation_required {
                println!("Run command: {}?", cmd.command);
                println!("[y/N]");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() != "y" {
                    return Err("Aborted.".to_string());
                }
            }
            
            match process.status() {
                Ok(status) => {
                    if !status.success() {
                        return Err(format!("Command failed with status: {}", status));
                    }
                }
                Err(e) => {
                    return Err(format!("Failed to execute command: {}", e));
                }
            }
            Ok(())
        }
        None => Err(format!("Command '{}' not found.", alias)),
    }
}

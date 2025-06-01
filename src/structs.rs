use clap::{Subcommand, Parser};

#[derive(Parser, Debug)]
#[command(name = "")]
pub struct KeyloggerCommands {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Starts key listener
    StartKeyListener,
    /// Stops key listener
    StopKeyListener,
    /// Reveals keylogger logs
    ShowLogs,
    /// Quit Program
    QuitProgram,
}
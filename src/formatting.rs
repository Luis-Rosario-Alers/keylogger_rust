use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::stdout;

pub fn update_process_header(process_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();

    stdout
        .execute(SavePosition)?
        .execute(MoveTo(0, 2))?
        .execute(Clear(ClearType::CurrentLine))?
        .execute(SetForegroundColor(Color::Cyan))?
        .execute(Print("📱 Current Process: "))?
        .execute(SetForegroundColor(Color::Green))?
        .execute(Print(process_name))?
        .execute(ResetColor)?
        .execute(RestorePosition)?;

    Ok(())
}

pub fn update_status_header(status: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();

    stdout
        .execute(SavePosition)?
        .execute(MoveTo(0, 3))?
        .execute(Clear(ClearType::CurrentLine))?
        .execute(SetForegroundColor(Color::Cyan))?
        .execute(Print("Status: "))?
        .execute(SetForegroundColor(Color::Green))?
        .execute(Print(status))?
        .execute(ResetColor)?
        .execute(RestorePosition)?;

    Ok(())
}

pub fn initialize_header() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    stdout
        .execute(Clear(ClearType::All))?
        .execute(MoveTo(0, 0))?
        .execute(SetBackgroundColor(Color::Blue))?
        .execute(SetForegroundColor(Color::White))?
        .execute(Print(format!("🎧 Rust Keylogger - Started: {} - User: [PLACEHOLDER]", timestamp)))?
        .execute(ResetColor)?
        .execute(Print("\n"))?
        .execute(Print("═".repeat(80)))?
        .execute(Print("\n📱 Current Process: Detecting..."))?
        .execute(Print("\nStatus: 🧏 Listening"))?
        .execute(Print("\n"))?
        .execute(Print("─".repeat(80)))?
        .execute(Print("\n"))?;

    Ok(())
}

pub fn clear_screen() -> Result<(), Box<dyn std::error::Error>> {
    stdout().execute(Clear(ClearType::All))?;
    Ok(())
}

pub fn clear_current_line() -> Result<(), Box<dyn std::error::Error>> {
    stdout().execute(Clear(ClearType::CurrentLine))?;
    Ok(())
}

pub fn clear_to_end_of_line() -> Result<(), Box<dyn std::error::Error>> {
    stdout().execute(Clear(ClearType::UntilNewLine))?;
    Ok(())
}
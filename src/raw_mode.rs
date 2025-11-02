use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;

pub struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
    //
    // pub fn enable() -> io::Result<Self> {
    //     enable_raw_mode()?;
    //     Ok(Self)
    // }
    //
    // pub fn disable() -> io::Result<Self> {
    //     disable_raw_mode()?;
    //     Ok(Self)
    // }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // Disable raw mode — errors ignored since we can’t return from Drop
        let _ = disable_raw_mode();
    }
}

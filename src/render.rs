use crossterm::{
    cursor, queue,
    style::{Attribute, Color, SetAttribute, SetBackgroundColor, SetForegroundColor, ResetColor},
    terminal::{Clear, ClearType},
};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use crate::navigator::Entry;

const MAX_VISIBLE: usize = 15;

pub struct Renderer {
    tty: File,
    previous_lines: usize,
    flash: Option<String>,
}

macro_rules! crlf {
    ($w:expr) => { write!($w, "\r\n") };
}

impl Renderer {
    pub fn new() -> io::Result<Self> {
        let tty = fs::OpenOptions::new().write(true).open("/dev/tty")?;
        Ok(Self { tty, previous_lines: 0, flash: None })
    }

    pub fn render(
        &mut self,
        cwd: &Path,
        entries: &[Entry],
        selected: usize,
        query: &str,
        show_hidden: bool,
        show_files: bool,
        scroll_offset: usize,
    ) -> io::Result<()> {
        if self.previous_lines > 0 {
            queue!(self.tty, cursor::MoveUp(self.previous_lines as u16), cursor::MoveToColumn(0))?;
        }

        let mut lines: usize = 0;

        // Header
        let display = format_path(cwd);
        queue!(self.tty, Clear(ClearType::CurrentLine), SetForegroundColor(Color::Blue), SetAttribute(Attribute::Bold))?;
        write!(self.tty, "  {}", display)?;
        queue!(self.tty, SetAttribute(Attribute::Reset), ResetColor)?;
        if show_hidden {
            queue!(self.tty, SetForegroundColor(Color::DarkYellow))?;
            write!(self.tty, " [H]")?;
            queue!(self.tty, ResetColor)?;
        }
        if show_files {
            queue!(self.tty, SetForegroundColor(Color::DarkYellow))?;
            write!(self.tty, " [F]")?;
            queue!(self.tty, ResetColor)?;
        }
        crlf!(self.tty)?;
        lines += 1;

        // Search bar
        if !query.is_empty() {
            queue!(self.tty, Clear(ClearType::CurrentLine))?;
            write!(self.tty, "  > {}", query)?;
            crlf!(self.tty)?;
            lines += 1;
        }

        if entries.is_empty() {
            queue!(self.tty, Clear(ClearType::CurrentLine), SetForegroundColor(Color::DarkGrey))?;
            write!(self.tty, "  (empty)")?;
            queue!(self.tty, ResetColor)?;
            crlf!(self.tty)?;
            lines += 1;
        } else {
            let total = entries.len();
            let visible = total.min(MAX_VISIBLE);
            let end = (scroll_offset + visible).min(total);

            if scroll_offset > 0 {
                queue!(self.tty, Clear(ClearType::CurrentLine), SetForegroundColor(Color::DarkGrey))?;
                write!(self.tty, "  ({} more)", scroll_offset)?;
                queue!(self.tty, ResetColor)?;
                crlf!(self.tty)?;
                lines += 1;
            }

            for i in scroll_offset..end {
                queue!(self.tty, Clear(ClearType::CurrentLine))?;

                let entry = &entries[i];
                let suffix = if entry.is_dir { "/" } else { "" };

                if i == selected {
                    queue!(self.tty, SetBackgroundColor(Color::White), SetForegroundColor(Color::Black))?;
                    write!(self.tty, " > {}{}", entry.name, suffix)?;
                    queue!(self.tty, SetBackgroundColor(Color::Reset), ResetColor)?;
                } else if entry.is_dir {
                    queue!(self.tty, SetForegroundColor(Color::Cyan))?;
                    write!(self.tty, "   {}{}", entry.name, suffix)?;
                    queue!(self.tty, ResetColor)?;
                } else {
                    queue!(self.tty, SetForegroundColor(Color::DarkGrey))?;
                    write!(self.tty, "   {}", entry.name)?;
                    queue!(self.tty, ResetColor)?;
                }

                crlf!(self.tty)?;
                lines += 1;
            }

            if end < total {
                queue!(self.tty, Clear(ClearType::CurrentLine), SetForegroundColor(Color::DarkGrey))?;
                write!(self.tty, "  ({} more)", total - end)?;
                queue!(self.tty, ResetColor)?;
                crlf!(self.tty)?;
                lines += 1;
            }
        }

        // Flash message
        if let Some(msg) = self.flash.take() {
            queue!(self.tty, Clear(ClearType::CurrentLine), SetForegroundColor(Color::Green))?;
            write!(self.tty, "  {}", msg)?;
            queue!(self.tty, ResetColor)?;
            crlf!(self.tty)?;
            lines += 1;
        }

        // Footer
        queue!(self.tty, Clear(ClearType::CurrentLine), SetForegroundColor(Color::DarkGrey))?;
        write!(self.tty, "  enter:cd  tab:here  Y:copy  ^F:files  arrows:navigate  esc:quit")?;
        queue!(self.tty, ResetColor)?;
        crlf!(self.tty)?;
        lines += 1;

        self.tty.flush()?;

        // Clear leftover lines
        if lines < self.previous_lines {
            let extra = self.previous_lines - lines;
            for _ in 0..extra {
                queue!(self.tty, Clear(ClearType::CurrentLine))?;
                crlf!(self.tty)?;
            }
            queue!(self.tty, cursor::MoveUp(extra as u16))?;
            self.tty.flush()?;
        }

        self.previous_lines = lines;
        Ok(())
    }

    pub fn set_flash(&mut self, msg: &str) {
        self.flash = Some(msg.to_string());
    }

    pub fn cleanup(&mut self) -> io::Result<()> {
        if self.previous_lines > 0 {
            queue!(self.tty, cursor::MoveUp(self.previous_lines as u16), cursor::MoveToColumn(0))?;
            for _ in 0..self.previous_lines {
                queue!(self.tty, Clear(ClearType::CurrentLine))?;
                crlf!(self.tty)?;
            }
            queue!(self.tty, cursor::MoveUp(self.previous_lines as u16), cursor::MoveToColumn(0))?;
            self.tty.flush()?;
        }
        self.previous_lines = 0;
        Ok(())
    }
}

fn format_path(path: &Path) -> String {
    if let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

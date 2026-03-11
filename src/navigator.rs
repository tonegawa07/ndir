use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::fuzzy::FuzzyFilter;
use crate::render::Renderer;

const MAX_VISIBLE: usize = 15;

#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
}

pub enum NavigationResult {
    Selected(PathBuf),
    Cancelled,
}

pub fn run(start_dir: PathBuf) -> io::Result<NavigationResult> {
    let mut cwd = fs::canonicalize(&start_dir)?;
    let mut selected: usize = 0;
    let mut query = String::new();
    let mut show_hidden = false;
    let mut show_files = false;
    let mut scroll_offset: usize = 0;
    let fuzzy = FuzzyFilter::new();
    let mut renderer = Renderer::new()?;

    terminal::enable_raw_mode()?;

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        default_hook(info);
    }));

    let result = event_loop(&mut cwd, &mut selected, &mut query, &mut show_hidden, &mut show_files, &mut scroll_offset, &fuzzy, &mut renderer);

    let _ = renderer.cleanup();
    let _ = terminal::disable_raw_mode();
    let _ = std::panic::take_hook();

    result
}

fn event_loop(
    cwd: &mut PathBuf,
    selected: &mut usize,
    query: &mut String,
    show_hidden: &mut bool,
    show_files: &mut bool,
    scroll_offset: &mut usize,
    fuzzy: &FuzzyFilter,
    renderer: &mut Renderer,
) -> io::Result<NavigationResult> {
    loop {
        let all = read_entries(cwd, *show_hidden, *show_files);
        let filtered = filter_entries(&all, query, fuzzy);

        // Clamp
        if filtered.is_empty() {
            *selected = 0;
        } else if *selected >= filtered.len() {
            *selected = filtered.len() - 1;
        }
        adjust_scroll(filtered.len(), *selected, scroll_offset);

        renderer.render(cwd, &filtered, *selected, query, *show_hidden, *show_files, *scroll_offset)?;

        if let Event::Key(key) = event::read()? {
            match handle_key(key, cwd, selected, query, show_hidden, show_files, scroll_offset, &filtered) {
                Action::Continue => {}
                Action::Accept(path) => return Ok(NavigationResult::Selected(path)),
                Action::Cancel => return Ok(NavigationResult::Cancelled),
                Action::CopyPath(path) => {
                    copy_to_clipboard(&path.display().to_string());
                    renderer.set_flash("Copied!");
                }
            }
        }
    }
}

enum Action {
    Continue,
    Accept(PathBuf),
    Cancel,
    CopyPath(PathBuf),
}

fn handle_key(
    key: KeyEvent,
    cwd: &mut PathBuf,
    selected: &mut usize,
    query: &mut String,
    show_hidden: &mut bool,
    show_files: &mut bool,
    scroll_offset: &mut usize,
    filtered: &[Entry],
) -> Action {
    let total = filtered.len();

    match key.code {
        KeyCode::Esc => Action::Cancel,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Cancel,

        // Up
        KeyCode::Up => {
            if total > 0 { *selected = if *selected == 0 { total - 1 } else { *selected - 1 }; }
            Action::Continue
        }
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if total > 0 { *selected = if *selected == 0 { total - 1 } else { *selected - 1 }; }
            Action::Continue
        }

        // Down
        KeyCode::Down => {
            if total > 0 { *selected = (*selected + 1) % total; }
            Action::Continue
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if total > 0 { *selected = (*selected + 1) % total; }
            Action::Continue
        }

        // Enter: accept selected directory (ignore files)
        KeyCode::Enter => {
            if total > 0 && *selected < total && filtered[*selected].is_dir {
                let target = cwd.join(&filtered[*selected].name);
                if let Ok(canonical) = fs::canonicalize(&target) {
                    return Action::Accept(canonical);
                }
            }
            Action::Continue
        }

        // Right: navigate into selected directory (ignore files)
        KeyCode::Right => {
            if total > 0 && *selected < total && filtered[*selected].is_dir {
                let new_path = cwd.join(&filtered[*selected].name);
                if let Ok(canonical) = fs::canonicalize(&new_path) {
                    *cwd = canonical;
                    *selected = 0;
                    query.clear();
                    *scroll_offset = 0;
                }
            }
            Action::Continue
        }

        // Left: parent directory
        KeyCode::Left => {
            let current_name = cwd.file_name().map(|n| n.to_string_lossy().to_string());
            if let Some(parent) = cwd.parent() {
                let parent = parent.to_path_buf();
                if parent != *cwd {
                    *cwd = parent;
                    query.clear();
                    *scroll_offset = 0;
                    if let Some(name) = current_name {
                        let entries = read_entries(cwd, *show_hidden, *show_files);
                        *selected = entries.iter().position(|e| e.name == name).unwrap_or(0);
                    } else {
                        *selected = 0;
                    }
                }
            }
            Action::Continue
        }

        // Tab: cd to current directory
        KeyCode::Tab => Action::Accept(cwd.clone()),

        // Copy path
        KeyCode::Char('Y') => {
            if total > 0 && *selected < total {
                let target = cwd.join(&filtered[*selected].name);
                if let Ok(canonical) = fs::canonicalize(&target) {
                    return Action::CopyPath(canonical);
                }
            }
            Action::Continue
        }

        // Toggle hidden
        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            *show_hidden = !*show_hidden;
            query.clear();
            *selected = 0;
            *scroll_offset = 0;
            Action::Continue
        }

        // Toggle files
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            *show_files = !*show_files;
            query.clear();
            *selected = 0;
            *scroll_offset = 0;
            Action::Continue
        }

        KeyCode::Backspace => {
            query.pop();
            *selected = 0;
            *scroll_offset = 0;
            Action::Continue
        }

        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            query.push(c);
            *selected = 0;
            *scroll_offset = 0;
            Action::Continue
        }

        _ => Action::Continue,
    }
}

fn read_entries(dir: &Path, show_hidden: bool, show_files: bool) -> Vec<Entry> {
    let Ok(rd) = fs::read_dir(dir) else { return Vec::new() };

    let mut dirs: Vec<Entry> = Vec::new();
    let mut files: Vec<Entry> = Vec::new();

    for e in rd.filter_map(|e| e.ok()) {
        let name = e.file_name().to_string_lossy().to_string();
        if !show_hidden && name.starts_with('.') { continue; }
        let is_dir = e.path().is_dir();
        if is_dir {
            dirs.push(Entry { name, is_dir: true });
        } else if show_files {
            files.push(Entry { name, is_dir: false });
        }
    }

    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    dirs.extend(files);
    dirs
}

fn filter_entries(entries: &[Entry], query: &str, fuzzy: &FuzzyFilter) -> Vec<Entry> {
    if query.is_empty() {
        return entries.to_vec();
    }
    let names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
    let matches = fuzzy.filter(query, &names);
    matches.into_iter().map(|(i, _)| entries[i].clone()).collect()
}

fn copy_to_clipboard(text: &str) {
    if cfg!(target_os = "macos") {
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .ok();
        if let Some(ref mut c) = child {
            if let Some(ref mut stdin) = c.stdin {
                let _ = io::Write::write_all(stdin, text.as_bytes());
            }
            let _ = c.wait();
        }
    } else {
        // Linux: try xclip, then xsel
        let mut child = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .or_else(|_| {
                Command::new("xsel")
                    .args(["--clipboard", "--input"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
            })
            .ok();
        if let Some(ref mut c) = child {
            if let Some(ref mut stdin) = c.stdin {
                let _ = io::Write::write_all(stdin, text.as_bytes());
            }
            let _ = c.wait();
        }
    }
}

fn adjust_scroll(total: usize, selected: usize, offset: &mut usize) {
    if total <= MAX_VISIBLE {
        *offset = 0;
    } else if selected < *offset {
        *offset = selected;
    } else if selected >= *offset + MAX_VISIBLE {
        *offset = selected - MAX_VISIBLE + 1;
    }
}

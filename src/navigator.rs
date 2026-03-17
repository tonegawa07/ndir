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

pub struct NavState {
    pub cwd: PathBuf,
    pub selected: usize,
    pub query: String,
    pub show_hidden: bool,
    pub show_files: bool,
    pub scroll_offset: usize,
}

pub enum NavigationResult {
    Selected(PathBuf),
    Cancelled,
}

pub fn run(start_dir: PathBuf) -> io::Result<NavigationResult> {
    let mut state = NavState {
        cwd: fs::canonicalize(&start_dir)?,
        selected: 0,
        query: String::new(),
        show_hidden: false,
        show_files: false,
        scroll_offset: 0,
    };
    let fuzzy = FuzzyFilter::new();
    let mut renderer = Renderer::new()?;

    terminal::enable_raw_mode()?;

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        default_hook(info);
    }));

    let result = event_loop(&mut state, &fuzzy, &mut renderer);

    let _ = renderer.cleanup();
    let _ = terminal::disable_raw_mode();
    let _ = std::panic::take_hook();

    result
}

fn event_loop(
    state: &mut NavState,
    fuzzy: &FuzzyFilter,
    renderer: &mut Renderer,
) -> io::Result<NavigationResult> {
    loop {
        let all = read_entries(&state.cwd, state.show_hidden, state.show_files);
        let filtered = filter_entries(&all, &state.query, fuzzy);

        // Clamp
        if filtered.is_empty() {
            state.selected = 0;
        } else if state.selected >= filtered.len() {
            state.selected = filtered.len() - 1;
        }
        adjust_scroll(filtered.len(), state.selected, &mut state.scroll_offset);

        renderer.render(state, &filtered)?;

        if let Event::Key(key) = event::read()? {
            match handle_key(key, state, &filtered) {
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

fn handle_key(key: KeyEvent, state: &mut NavState, filtered: &[Entry]) -> Action {
    let total = filtered.len();

    match key.code {
        KeyCode::Esc => Action::Cancel,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Cancel,

        // Up
        KeyCode::Up => {
            if total > 0 {
                state.selected = if state.selected == 0 {
                    total - 1
                } else {
                    state.selected - 1
                };
            }
            Action::Continue
        }
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if total > 0 {
                state.selected = if state.selected == 0 {
                    total - 1
                } else {
                    state.selected - 1
                };
            }
            Action::Continue
        }

        // Down
        KeyCode::Down => {
            if total > 0 {
                state.selected = (state.selected + 1) % total;
            }
            Action::Continue
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if total > 0 {
                state.selected = (state.selected + 1) % total;
            }
            Action::Continue
        }

        // Enter: accept selected directory (ignore files)
        KeyCode::Enter => {
            if total > 0 && state.selected < total && filtered[state.selected].is_dir {
                let target = state.cwd.join(&filtered[state.selected].name);
                if let Ok(canonical) = fs::canonicalize(&target) {
                    return Action::Accept(canonical);
                }
            }
            Action::Continue
        }

        // Right: navigate into selected directory (ignore files)
        KeyCode::Right => {
            if total > 0 && state.selected < total && filtered[state.selected].is_dir {
                let new_path = state.cwd.join(&filtered[state.selected].name);
                if let Ok(canonical) = fs::canonicalize(&new_path) {
                    state.cwd = canonical;
                    state.selected = 0;
                    state.query.clear();
                    state.scroll_offset = 0;
                }
            }
            Action::Continue
        }

        // Left: parent directory
        KeyCode::Left => {
            let current_name = state
                .cwd
                .file_name()
                .map(|n| n.to_string_lossy().to_string());
            if let Some(parent) = state.cwd.parent() {
                let parent = parent.to_path_buf();
                if parent != state.cwd {
                    state.cwd = parent;
                    state.query.clear();
                    state.scroll_offset = 0;
                    if let Some(name) = current_name {
                        let entries = read_entries(&state.cwd, state.show_hidden, state.show_files);
                        state.selected = entries.iter().position(|e| e.name == name).unwrap_or(0);
                    } else {
                        state.selected = 0;
                    }
                }
            }
            Action::Continue
        }

        // Tab: cd to current directory
        KeyCode::Tab => Action::Accept(state.cwd.clone()),

        // Copy path
        KeyCode::Char('Y') => {
            if total > 0 && state.selected < total {
                let target = state.cwd.join(&filtered[state.selected].name);
                if let Ok(canonical) = fs::canonicalize(&target) {
                    return Action::CopyPath(canonical);
                }
            }
            Action::Continue
        }

        // Toggle hidden
        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.show_hidden = !state.show_hidden;
            state.query.clear();
            state.selected = 0;
            state.scroll_offset = 0;
            Action::Continue
        }

        // Toggle files
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.show_files = !state.show_files;
            state.query.clear();
            state.selected = 0;
            state.scroll_offset = 0;
            Action::Continue
        }

        KeyCode::Backspace => {
            state.query.pop();
            state.selected = 0;
            state.scroll_offset = 0;
            Action::Continue
        }

        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.query.push(c);
            state.selected = 0;
            state.scroll_offset = 0;
            Action::Continue
        }

        _ => Action::Continue,
    }
}

fn read_entries(dir: &Path, show_hidden: bool, show_files: bool) -> Vec<Entry> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut dirs: Vec<Entry> = Vec::new();
    let mut files: Vec<Entry> = Vec::new();

    for e in rd.filter_map(|e| e.ok()) {
        let name = e.file_name().to_string_lossy().to_string();
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        let is_dir = e.path().is_dir();
        if is_dir {
            dirs.push(Entry { name, is_dir: true });
        } else if show_files {
            files.push(Entry {
                name,
                is_dir: false,
            });
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
    matches
        .into_iter()
        .map(|(i, _)| entries[i].clone())
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    fn make_state(cwd: PathBuf) -> NavState {
        NavState {
            cwd,
            selected: 0,
            query: String::new(),
            show_hidden: false,
            show_files: false,
            scroll_offset: 0,
        }
    }

    fn sample_entries() -> Vec<Entry> {
        vec![
            Entry {
                name: "alpha".into(),
                is_dir: true,
            },
            Entry {
                name: "beta".into(),
                is_dir: true,
            },
            Entry {
                name: "gamma".into(),
                is_dir: false,
            },
        ]
    }

    // ── read_entries ──

    #[test]
    fn read_entries_lists_dirs_only_by_default() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("dir_a")).unwrap();
        fs::create_dir(tmp.path().join("dir_b")).unwrap();
        fs::write(tmp.path().join("file.txt"), "").unwrap();

        let entries = read_entries(tmp.path(), false, false);
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.is_dir));
    }

    #[test]
    fn read_entries_includes_files_when_show_files() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("dir_a")).unwrap();
        fs::write(tmp.path().join("file.txt"), "").unwrap();

        let entries = read_entries(tmp.path(), false, true);
        assert_eq!(entries.len(), 2);
        // dirs come first
        assert!(entries[0].is_dir);
        assert!(!entries[1].is_dir);
    }

    #[test]
    fn read_entries_hides_dotfiles_by_default() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".hidden")).unwrap();
        fs::create_dir(tmp.path().join("visible")).unwrap();

        let entries = read_entries(tmp.path(), false, false);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "visible");
    }

    #[test]
    fn read_entries_shows_hidden_when_enabled() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join(".hidden")).unwrap();
        fs::create_dir(tmp.path().join("visible")).unwrap();

        let entries = read_entries(tmp.path(), true, false);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn read_entries_sorted_case_insensitive() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("Zebra")).unwrap();
        fs::create_dir(tmp.path().join("apple")).unwrap();
        fs::create_dir(tmp.path().join("Banana")).unwrap();

        let entries = read_entries(tmp.path(), false, false);
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["apple", "Banana", "Zebra"]);
    }

    #[test]
    fn read_entries_nonexistent_dir_returns_empty() {
        let entries = read_entries(Path::new("/nonexistent_path_12345"), false, false);
        assert!(entries.is_empty());
    }

    // ── filter_entries ──

    #[test]
    fn filter_entries_empty_query_returns_all() {
        let fuzzy = FuzzyFilter::new();
        let entries = sample_entries();
        let result = filter_entries(&entries, "", &fuzzy);
        assert_eq!(result.len(), entries.len());
    }

    #[test]
    fn filter_entries_narrows_results() {
        let fuzzy = FuzzyFilter::new();
        let entries = sample_entries();
        let result = filter_entries(&entries, "alp", &fuzzy);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "alpha");
    }

    #[test]
    fn filter_entries_preserves_is_dir() {
        let fuzzy = FuzzyFilter::new();
        let entries = sample_entries();
        let result = filter_entries(&entries, "gamma", &fuzzy);
        assert_eq!(result.len(), 1);
        assert!(!result[0].is_dir);
    }

    // ── adjust_scroll ──

    #[test]
    fn adjust_scroll_no_scroll_when_fits() {
        let mut offset = 5;
        adjust_scroll(10, 3, &mut offset);
        assert_eq!(offset, 0);
    }

    #[test]
    fn adjust_scroll_scrolls_up_when_selected_above() {
        let mut offset = 10;
        adjust_scroll(30, 5, &mut offset);
        assert_eq!(offset, 5);
    }

    #[test]
    fn adjust_scroll_scrolls_down_when_selected_below() {
        let mut offset = 0;
        adjust_scroll(30, 20, &mut offset);
        assert_eq!(offset, 20 - MAX_VISIBLE + 1);
    }

    #[test]
    fn adjust_scroll_keeps_offset_when_visible() {
        let mut offset = 5;
        adjust_scroll(30, 10, &mut offset);
        assert_eq!(offset, 5);
    }

    // ── handle_key ──

    #[test]
    fn handle_key_esc_cancels() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        let entries = sample_entries();
        let key = make_key(KeyCode::Esc, KeyModifiers::NONE);
        assert!(matches!(
            handle_key(key, &mut state, &entries),
            Action::Cancel
        ));
    }

    #[test]
    fn handle_key_ctrl_c_cancels() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        let entries = sample_entries();
        let key = make_key(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(matches!(
            handle_key(key, &mut state, &entries),
            Action::Cancel
        ));
    }

    #[test]
    fn handle_key_down_increments_selected() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        let entries = sample_entries();
        let key = make_key(KeyCode::Down, KeyModifiers::NONE);
        handle_key(key, &mut state, &entries);
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn handle_key_down_wraps_around() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        state.selected = 2;
        let entries = sample_entries();
        let key = make_key(KeyCode::Down, KeyModifiers::NONE);
        handle_key(key, &mut state, &entries);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn handle_key_up_decrements_selected() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        state.selected = 2;
        let entries = sample_entries();
        let key = make_key(KeyCode::Up, KeyModifiers::NONE);
        handle_key(key, &mut state, &entries);
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn handle_key_up_wraps_to_bottom() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        let entries = sample_entries();
        let key = make_key(KeyCode::Up, KeyModifiers::NONE);
        handle_key(key, &mut state, &entries);
        assert_eq!(state.selected, 2);
    }

    #[test]
    fn handle_key_char_appends_to_query() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        let entries = sample_entries();
        let key = make_key(KeyCode::Char('a'), KeyModifiers::NONE);
        handle_key(key, &mut state, &entries);
        assert_eq!(state.query, "a");
    }

    #[test]
    fn handle_key_backspace_removes_from_query() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        state.query = "abc".into();
        let entries = sample_entries();
        let key = make_key(KeyCode::Backspace, KeyModifiers::NONE);
        handle_key(key, &mut state, &entries);
        assert_eq!(state.query, "ab");
    }

    #[test]
    fn handle_key_ctrl_h_toggles_hidden() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        let entries = sample_entries();
        let key = make_key(KeyCode::Char('h'), KeyModifiers::CONTROL);
        handle_key(key, &mut state, &entries);
        assert!(state.show_hidden);
        handle_key(key, &mut state, &entries);
        assert!(!state.show_hidden);
    }

    #[test]
    fn handle_key_ctrl_f_toggles_files() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        let entries = sample_entries();
        let key = make_key(KeyCode::Char('f'), KeyModifiers::CONTROL);
        handle_key(key, &mut state, &entries);
        assert!(state.show_files);
    }

    #[test]
    fn handle_key_tab_accepts_cwd() {
        let tmp = TempDir::new().unwrap();
        let cwd = tmp.path().to_path_buf();
        let mut state = make_state(cwd.clone());
        let entries = sample_entries();
        let key = make_key(KeyCode::Tab, KeyModifiers::NONE);
        let action = handle_key(key, &mut state, &entries);
        assert!(matches!(action, Action::Accept(p) if p == cwd));
    }

    #[test]
    fn handle_key_enter_on_dir_accepts() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        let entries = vec![Entry {
            name: "subdir".into(),
            is_dir: true,
        }];
        let key = make_key(KeyCode::Enter, KeyModifiers::NONE);
        let action = handle_key(key, &mut state, &entries);
        assert!(matches!(action, Action::Accept(_)));
    }

    #[test]
    fn handle_key_enter_on_file_continues() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("file.txt"), "").unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        let entries = vec![Entry {
            name: "file.txt".into(),
            is_dir: false,
        }];
        let key = make_key(KeyCode::Enter, KeyModifiers::NONE);
        let action = handle_key(key, &mut state, &entries);
        assert!(matches!(action, Action::Continue));
    }

    #[test]
    fn handle_key_right_navigates_into_dir() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("child");
        fs::create_dir(&sub).unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        let entries = vec![Entry {
            name: "child".into(),
            is_dir: true,
        }];
        let key = make_key(KeyCode::Right, KeyModifiers::NONE);
        handle_key(key, &mut state, &entries);
        assert_eq!(state.cwd, fs::canonicalize(&sub).unwrap());
        assert_eq!(state.selected, 0);
        assert!(state.query.is_empty());
    }

    #[test]
    fn handle_key_left_goes_to_parent() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("child");
        fs::create_dir(&sub).unwrap();
        let canonical_sub = fs::canonicalize(&sub).unwrap();
        let canonical_tmp = fs::canonicalize(tmp.path()).unwrap();
        let mut state = make_state(canonical_sub);
        let entries = sample_entries();
        let key = make_key(KeyCode::Left, KeyModifiers::NONE);
        handle_key(key, &mut state, &entries);
        assert_eq!(state.cwd, canonical_tmp);
    }

    #[test]
    fn handle_key_vim_ctrl_j_moves_down() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        let entries = sample_entries();
        let key = make_key(KeyCode::Char('j'), KeyModifiers::CONTROL);
        handle_key(key, &mut state, &entries);
        assert_eq!(state.selected, 1);
    }

    #[test]
    fn handle_key_vim_ctrl_k_moves_up() {
        let tmp = TempDir::new().unwrap();
        let mut state = make_state(tmp.path().to_path_buf());
        state.selected = 1;
        let entries = sample_entries();
        let key = make_key(KeyCode::Char('k'), KeyModifiers::CONTROL);
        handle_key(key, &mut state, &entries);
        assert_eq!(state.selected, 0);
    }
}

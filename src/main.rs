mod fuzzy;
mod navigator;
mod render;

use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    let arg = env::args().nth(1);

    if arg.as_deref() == Some("--version") {
        println!("ndir {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if arg.as_deref() == Some("--help") {
        println!("ndir - Inline arrow-key directory navigation for your shell");
        println!();
        println!("Usage: ndir [directory]");
        println!();
        println!("Options:");
        println!("  --init       Print shell setup script");
        println!("  --version    Print version");
        println!("  --help       Print this help");
        println!();
        println!("Keys:");
        println!("  ↑/↓          Move cursor");
        println!("  Enter        cd to selected directory");
        println!("  →            Browse into directory");
        println!("  ←            Go back to parent");
        println!("  Tab          cd to current directory");
        println!("  Esc          Cancel");
        println!("  Ctrl+H       Toggle hidden files");
        println!("  Ctrl+F       Toggle file display");
        println!("  Y            Copy selected path to clipboard");
        println!("  Type         Fuzzy filter");
        return;
    }

    if arg.as_deref() == Some("--init") {
        print!("{}", include_str!("../shell/ndir.zsh"));
        return;
    }

    let start_dir = arg
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    match navigator::run(start_dir) {
        Ok(navigator::NavigationResult::Selected(path)) => {
            println!("{}", path.display());
        }
        Ok(navigator::NavigationResult::Cancelled) => {
            process::exit(1);
        }
        Err(e) => {
            eprintln!("ndir: {}", e);
            process::exit(1);
        }
    }
}

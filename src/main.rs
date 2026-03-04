mod fuzzy;
mod navigator;
mod render;

use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    let arg = env::args().nth(1);

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

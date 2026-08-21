use std::collections::HashSet;

use crate::ui::Ui;

mod board;
mod game;
mod ui;

fn main() -> Result<(), ()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut play_white = true;
    let mut threads_next = false;
    let mut depth_next = false;
    let mut threads = 10;
    let mut depth = 5;
    let mut seen = HashSet::new();
    let mut force_white_bottom = false;

    for arg in args {
        if threads_next {
            threads = match usize::from_str_radix(&arg, 10) {
                Ok(threads) => threads,
                Err(_) => {
                    eprintln!("{} is not a number", arg);
                    return Err(());
                }
            };
            threads_next = false;
            continue;
        } else if depth_next {
            depth = match usize::from_str_radix(&arg, 10) {
                Ok(threads) => threads,
                Err(_) => {
                    eprintln!("{} is not a number", arg);
                    return Err(());
                }
            };
            depth_next = false;
            continue;
        }

        if seen.contains(&arg) {
            eprintln!("Duplicate argument {}", arg);
            return Err(());
        } else {
            seen.insert(arg.clone());
        }

        match &arg[..] {
            "-b" => play_white = false,
            "-t" => threads_next = true,
            "-d" => depth_next = true,
            "-f" => force_white_bottom = true,
            _ => {
                eprintln!("Unknown argument {}", arg);
                return Err(());
            }
        }
    }
    let white_on_bottom = play_white || force_white_bottom;

    Ui::new(depth, threads, white_on_bottom, play_white).start();
    Ok(())
}

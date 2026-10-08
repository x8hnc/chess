use std::collections::HashSet;

use crate::{tui::Tui, web::WebUI};

mod board;
mod game;
mod tui;
mod web;

// TODO: implement draw by insufficient material
// TODO: implement black on the bottom for web ui

struct Options {
    play_white: bool,
    thread_count: usize,
    depth: usize,
    force_white_bottom: bool,
    run_tui: bool,
    pvp: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            play_white: true,
            thread_count: 20,
            depth: 6,
            force_white_bottom: false,
            run_tui: false,
            pvp: false,
        }
    }
}

fn main() -> Result<(), ()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut options = Options::default();
    let mut threads_next = false;
    let mut depth_next = false;
    let mut seen = HashSet::new();

    for arg in args {
        if threads_next {
            options.thread_count = match arg.parse::<usize>() {
                Ok(threads) => threads,
                Err(_) => {
                    eprintln!("{} is not a number", arg);
                    return Err(());
                }
            };
            threads_next = false;
            continue;
        } else if depth_next {
            options.depth = match arg.parse::<usize>() {
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
            "-b" => options.play_white = false,
            "-t" => threads_next = true,
            "-d" => depth_next = true,
            "-f" => options.force_white_bottom = true,
            "-h" => options.run_tui = true,
            "-p" => options.pvp = true,
            _ => {
                eprintln!("Unknown argument {}", arg);
                return Err(());
            }
        }
    }

    options.force_white_bottom = options.play_white || options.force_white_bottom;

    if options.run_tui {
        Tui::new(options).start();
    } else {
        WebUI::new("127.0.0.1:8585", options).unwrap().start();
    }
    Ok(())
}

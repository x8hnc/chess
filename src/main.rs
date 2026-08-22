use std::collections::HashSet;

use crate::{tui::Tui, web::WebUI};

mod board;
mod game;
mod tui;
mod web;

// TODO: implement draw by repetition
// TODO: implement draw by insufficient material
// TODO: implement draw by 50 move rule
// TODO: implement black on the bottom for web ui

fn main() -> Result<(), ()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut play_white = true;
    let mut threads_next = false;
    let mut depth_next = false;
    let mut threads = 20;
    let mut depth = 6;
    let mut seen = HashSet::new();
    let mut force_white_bottom = false;
    let mut run_tui = false;

    for arg in args {
        if threads_next {
            threads = match arg.parse::<usize>() {
                Ok(threads) => threads,
                Err(_) => {
                    eprintln!("{} is not a number", arg);
                    return Err(());
                }
            };
            threads_next = false;
            continue;
        } else if depth_next {
            depth = match arg.parse::<usize>() {
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
            "-h" => run_tui = true,
            _ => {
                eprintln!("Unknown argument {}", arg);
                return Err(());
            }
        }
    }
    let white_on_bottom = play_white || force_white_bottom;
    if run_tui {
        // Tui::new(depth, threads, white_on_bottom, play_white).start();
        Tui::_from_fen("8/3r4/4r3/8/8/8/5k2/6nK w - - 0 1", depth, threads, white_on_bottom, play_white).unwrap().start();
    } else {
        WebUI::new(
            "127.0.0.1:8585",
            depth,
            threads,
            white_on_bottom,
            play_white,
        )
        .unwrap()
        .start();
    }
    Ok(())
}

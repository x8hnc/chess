use crate::ui::Ui;

mod board;
mod game;
mod ui;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let play_first = if args.contains(&String::from("-b")) {
        false
    } else {
        true
    };

    Ui::new().start(play_first);
}

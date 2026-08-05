use crate::ui::Ui;

mod board;
mod game;
mod ui;

fn main() {
    Ui::new().start();
}

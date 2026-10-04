//! An open yard, four lights sweeping it, and the far side. See `specs/`.

mod beams;
mod caught;
mod seen;
mod sweep_game;
mod walker;
mod wires;
mod yard;

use blitzkit::start;
use sweep_game::SweepGame;

/// Whether this run is only here to be photographed, for `refresh-screenshots`
/// in the project above.
pub fn staged() -> bool {
    std::env::args().any(|arg| arg == "--screenshot")
}

fn main() {
    start("securitysweep", Box::new(SweepGame::new()));
}

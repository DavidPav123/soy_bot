mod soy_bot;
mod soy_player;
use crate::soy_bot::SoyBot;
use rust_sc2::prelude::*;

fn main() -> SC2Result<()> {
    let mut bot = SoyBot::default();
    run_vs_computer(
        &mut bot,
        Computer::new(Race::Random, Difficulty::VeryEasy, None),
        "PylonAIE_v2",
        LaunchOptions {
            sc2_version: Default::default(),
            save_replay_as: Default::default(),
            realtime: true,
        },
    )
}

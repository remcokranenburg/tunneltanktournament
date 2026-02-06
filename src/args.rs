use bevy::prelude::*;
use clap::Parser;

#[derive(Parser, Debug, Resource)]
pub struct Args {
    /// runs the game in synctest mode
    #[clap(long)]
    pub synctest: bool,

    /// runs the game in local mode
    #[clap(long)]
    pub local: bool,

    /// input delay in frames
    #[clap(long, default_value_t = 2)]
    pub input_delay: usize,

    /// enables debug mode
    #[clap(long)]
    pub debug: bool,
}

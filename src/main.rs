mod cli;

use clap::Parser;
use cli::{Opts, start_cli};

fn main() {
    let opts = Opts::parse();

    start_cli(opts);
}
extern crate clap;
#[macro_use]
extern crate bitflags;

pub(crate) mod host;
pub(crate) mod opts;

use clap::Clap;
use opts::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: opts::Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Host(opts) => {
            host::run(opts)
        }
    }
}

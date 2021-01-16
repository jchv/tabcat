
use clap::{Clap,ArgEnum};
use std::str::FromStr;

#[derive(Clap)]
#[clap(version = "1.0", author = "John Chadwick <john@jchw.io>")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap)]
pub enum SubCommand {
    Host(HostOpts),
}

#[derive(Debug,ArgEnum)]
pub enum HostDriver{
    #[cfg(feature = "x11")]
    X11,
}

impl FromStr for HostDriver {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "x11")]
            "x11" => Ok(Self::X11),
            _ => Err("no match"),
        }
    }
}

#[derive(Clap)]
pub struct HostOpts {
    pub driver: HostDriver,
}

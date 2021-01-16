mod event;

use std::{sync::mpsc::sync_channel, thread};

use event::*;

use crate::opts::{HostDriver, HostOpts};

#[cfg(feature = "x11")]
pub(crate) mod x11;

pub(crate) fn run(opts: HostOpts) -> Result<(), Box<dyn std::error::Error>> {
    let (send, recv) = sync_channel(32);

    // Start host driver thread.
    thread::spawn(move|| {
        let result = match opts.driver {
            #[cfg(feature = "x11")]
            HostDriver::X11 => x11::run(send),
        };

        match result {
            Ok(_) => {},
            Err(err) => {
                println!("host driver exited due to error: {}", err.to_string());
            }
        }
    });
    
    loop {
        match recv.recv() {
            Ok(info) => {
                println!("got event: {:?}", info);
            },
            Err(_) => {
                // If the sender disconnects that means the host thread has
                // closed.
                return Ok(());
            },
        }
    }
}
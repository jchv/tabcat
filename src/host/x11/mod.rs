extern crate x11rb;

use std::sync::mpsc::SyncSender;

use x11rb::connection::Connection;
use x11rb::protocol::xinput;

use super::PointerInfo;

pub(crate) fn run(_send: SyncSender<PointerInfo>) -> Result<(), Box<dyn std::error::Error>> {
    let (conn, screen_num) = x11rb::connect(None).unwrap();

    xinput::xi_query_version(&conn, 2, 2)?;

    let screen = &conn.setup().roots[screen_num];
    xinput::xi_select_events(&conn, screen.root, &[
        xinput::EventMask{deviceid: 0, mask: vec![
            (xinput::XIEventMask::BUTTON_PRESS|xinput::XIEventMask::MOTION).into(),
        ]},
    ])?;

    conn.flush()?;

    loop {
        match conn.wait_for_event()? {
            x11rb::protocol::Event::Unknown(_) => {
                println!("x11 host: unknown event");
            }
            x11rb::protocol::Event::Error(err) => {
                println!("x11 host: error: {:?}", err);
            }
            x11rb::protocol::Event::XinputMotion(_event) => {
                // TODO(jchw): handle event
            }
            x11rb::protocol::Event::XinputButtonPress(_event) => {
                // TODO(jchw): handle event
            }
            x11rb::protocol::Event::XinputButtonRelease(_event) => {
                // TODO(jchw): handle event
            }
            x11rb::protocol::Event::XinputRawButtonPress(_event) => {
                // TODO(jchw): handle event
            }
            x11rb::protocol::Event::XinputRawButtonRelease(_event) => {
                // TODO(jchw): handle event
            }
            event => {
                println!("x11 host: unhandled: {:?}", event);
            }
        }
    }
}

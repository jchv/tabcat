extern crate x11rb;

use std::{collections::HashMap, sync::mpsc::SyncSender};

use x11rb::{atom_manager, connection::Connection};
use x11rb::protocol::xinput;

use super::PointerInfo;

atom_manager! {
    pub AtomCollection: AtomCollectionCookie {
        ABS_X: b"Abs X",
        ABS_Y: b"Abs Y",
        ABS_PRESSURE: b"Abs Pressure",
        ABS_TILT_X: b"Abs Tilt X",
        ABS_TILT_Y: b"Abs Tilt Y",
        ABS_WHEEL: b"Abs Wheel",
        REL_X: b"Rel X",
        REL_Y: b"Rel Y",
    }
}

#[derive(Debug)]
struct ValuatorRange {
    min: i64,
    max: i64,
}

impl ValuatorRange {
    fn convert_range(&self, value: i64, new_min: i64, new_max: i64) -> i64 {
        let value = value as i128;
        let old_min = self.min as i128;
        let old_max = self.max as i128;
        let old_size = old_max - old_min;
        let new_min = new_min as i128;
        let new_max = new_max as i128;
        let new_size = new_max - new_min;
        ((((value - old_min) * new_size) / old_size) + new_min) as i64
    }
}

#[derive(Debug)]
enum TabletValuator {
    AbsX(ValuatorRange),
    AbsY(ValuatorRange),
    AbsPressure(ValuatorRange),
    AbsTiltX(ValuatorRange),
    AbsTiltY(ValuatorRange),
    AbsWheel(ValuatorRange),
}

#[derive(Debug)]
enum TabletRole {
    Default,
    Eraser,
}

impl Default for TabletRole {
    fn default() -> Self { Self::Default }
}

#[derive(Copy, Clone, Debug, Default)]
struct TabletPointerState {
    x: i32,
    y: i32,
    pressure: i32,
    tilt_x: i32,
    tilt_y: i32,
    button: [bool; 5],
}

impl TabletPointerState {
    fn apply_event(&self, device: &TabletDevice, event: &xinput::ButtonPressEvent) -> TabletPointerState {
        let mut new_state = self.clone();
        new_state.x = event.root_x;
        new_state.y = event.root_y;
        let mask_iter = {
            event.valuator_mask.iter()
                .flat_map(|n| (0..32u32)
                    .map(move |x| (n & (1 << x)) != 0)
                )
                .enumerate()
                .filter(|(_i, b)| *b)
                .map(|(i, _b)| i as u16)
                .enumerate()
        };
        for (index, valuatorid) in mask_iter.into_iter() {
            if let (Some(valuator), Some(axis)) = (device.valuators.get(&valuatorid), event.axisvalues.get(index)) {
                match valuator {
                    TabletValuator::AbsPressure(range) => {
                        new_state.pressure = range.convert_range(fp3232_to_i64(axis), 0, 1024) as _;
                    }
                    TabletValuator::AbsTiltX(range) => {
                        new_state.tilt_x = range.convert_range(fp3232_to_i64(axis), -90, 90) as _;
                    }
                    TabletValuator::AbsTiltY(range) => {
                        new_state.tilt_y = range.convert_range(fp3232_to_i64(axis), -90, 90) as _;
                    }
                    _ => {}
                }
            }
        }
        new_state
    }
}

#[derive(Debug, Default)]
struct TabletDevice {
    valuators: HashMap<u16, TabletValuator>,
    role: TabletRole,
    state: TabletPointerState,
}

fn fp3232_to_i64(v: &xinput::Fp3232) -> i64 {
    return (((v.integral as u64) << 32) | (v.frac as u64)) as i64
}

fn make_valuator_range(valuator_class: &xinput::DeviceClassDataValuator) -> ValuatorRange {
    ValuatorRange {
        min: fp3232_to_i64(&valuator_class.min),
        max: fp3232_to_i64(&valuator_class.max),
    }
}

fn build_valuators_map(atoms: &AtomCollection, classes: &[xinput::DeviceClass]) -> HashMap<u16, TabletValuator> {
    let mut result = HashMap::new();
    for class in classes.iter() {
        match class.data {
            xinput::DeviceClassData::Valuator(valuator) => {
                if valuator.label == atoms.ABS_X {
                    result.insert(valuator.number, TabletValuator::AbsX(make_valuator_range(&valuator)));
                } else if valuator.label == atoms.ABS_Y {
                    result.insert(valuator.number, TabletValuator::AbsY(make_valuator_range(&valuator)));
                } else if valuator.label == atoms.ABS_PRESSURE {
                    result.insert(valuator.number, TabletValuator::AbsPressure(make_valuator_range(&valuator)));
                } else if valuator.label == atoms.ABS_TILT_X {
                    result.insert(valuator.number, TabletValuator::AbsTiltX(make_valuator_range(&valuator)));
                } else if valuator.label == atoms.ABS_TILT_Y {
                    result.insert(valuator.number, TabletValuator::AbsTiltY(make_valuator_range(&valuator)));
                } else if valuator.label == atoms.ABS_WHEEL {
                    result.insert(valuator.number, TabletValuator::AbsWheel(make_valuator_range(&valuator)));
                }
            }
            _ => {}
        }
    }
    result
}

fn is_tablet_device(atoms: &AtomCollection, classes: &[xinput::DeviceClass]) -> bool {
    classes.iter().any(|class| class.data.as_valuator().map_or(false, |valuator| valuator.label == atoms.ABS_PRESSURE))
}

fn setup_tablet_device(atoms: &AtomCollection, name: &str, classes: &[xinput::DeviceClass]) -> Option<TabletDevice> {
    if !is_tablet_device(atoms, classes) {
        None
    } else {
        let role = if name.to_ascii_lowercase().contains("eraser") {
            TabletRole::Eraser
        } else {
            TabletRole::Default
        };
        let valuators = build_valuators_map(atoms, classes);
        Some(TabletDevice{
            valuators,
            role,
            ..Default::default()
        })   
    }
}

pub(crate) fn run(_send: SyncSender<PointerInfo>) -> Result<(), Box<dyn std::error::Error>> {
    let mut tablets = HashMap::new();
    let (conn, screen_num) = x11rb::connect(None).unwrap();
    let atoms = AtomCollection::new(&conn)?.reply()?;

    // Setup XI2
    xinput::xi_query_version(&conn, 2, 2)?;
    let screen = &conn.setup().roots[screen_num];
    xinput::xi_select_events(&conn, screen.root, &[
        xinput::EventMask{deviceid: 0, mask: vec![
            (
                xinput::XIEventMask::BUTTON_PRESS | 
                xinput::XIEventMask::MOTION | 
                xinput::XIEventMask::DEVICE_CHANGED
            ).into(),
        ]},
    ])?;

    // Enumerate devices
    for device in xinput::xi_query_device(&conn, 0u16)?.reply()?.infos {
        match device.type_ {
            xinput::DeviceType::SLAVE_POINTER => {
                let name = String::from_utf8_lossy(&device.name);
                if let Some(tablet_device) = setup_tablet_device(&atoms, &name, &device.classes) {
                    println!("detected tablet (device={})", device.deviceid);
                    tablets.insert(device.deviceid, tablet_device);
                }
            }
            _ => {}
        }
    }

    conn.flush()?;

    loop {
        match conn.wait_for_event()? {
            x11rb::protocol::Event::Unknown(_) => {
                println!("x11 host: unknown event");
            }
            x11rb::protocol::Event::Error(err) => {
                println!("x11 host: error: {:?}", err);
            }
            x11rb::protocol::Event::XinputDeviceChanged(event) => {
                if let Some(tablet) = tablets.get_mut(&event.deviceid) {
                    println!("configured tablet (device={})", event.deviceid);
                    tablet.valuators = build_valuators_map(&atoms, &event.classes);
                }
            }
            x11rb::protocol::Event::XinputMotion(event) |
            x11rb::protocol::Event::XinputButtonPress(event) |
            x11rb::protocol::Event::XinputButtonRelease(event) => {
                if let Some(tablet) = tablets.get_mut(&event.deviceid) {
                    tablet.state = tablet.state.apply_event(&tablet, &event);
                }
            }
            event => {
                println!("x11 host: unhandled: {:?}", event);
            }
        }
    }
}

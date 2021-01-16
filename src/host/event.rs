bitflags! {
    pub(crate) struct PenFlags: u32 {
        const NONE = 0;
        const BARREL = 1;
        const INVERTED = 2;
        const ERASER = 4;
    }
}

bitflags! {
    pub(crate) struct PointerFlags: u32 {
        const NONE = 0x00;
        const NEW = 0x01;
        const IN_RANGE = 0x02;
        const IN_CONTACT = 0x04;
        const FIRST_BUTTON = 0x10;
        const SECOND_BUTTON = 0x20;
        const THIRD_BUTTON = 0x40;
        const FORTH_BUTTON = 0x80;
        const FIFTH_BUTTON = 0x100;
        const PRIMARY = 0x2000;
        const CONFIDENCE = 0x4000;
        const CANCELED = 0x8000;
        const DOWN = 0x10000;
        const UPDATE = 0x20000;
        const UP = 0x40000;
        const WHEEL = 0x80000;
        const HWHEEL = 0x100000;
    }
}

#[derive(Debug)]
pub(crate) struct Point<T> {
    pub(crate) x: T,
    pub(crate) y: T,
}

#[derive(Debug)]
pub(crate) struct Rect<T> {
    pub(crate) left: T,
    pub(crate) top: T,
    pub(crate) right: T,
    pub(crate) bottom: T,
}

#[derive(Debug)]
pub(crate) struct TouchInfo {
    pub(crate) contact: Rect<u32>,
    pub(crate) orientation: u16,
    pub(crate) pressure: u32,
}

#[derive(Debug)]
pub(crate) struct PenInfo {
    pub(crate) flags: PenFlags,
    pub(crate) pressure: u32,
    pub(crate) rotation: u32,
    pub(crate) tilt_x: u32,
    pub(crate) tilt_y: u32,
}

#[derive(Debug)]
pub(crate) enum PointerType {
    Touch(TouchInfo),
    Pen(PenInfo),
}

#[derive(Debug)]
pub(crate) struct PointerInfo {
    pub(crate) pointer_type: PointerType,
    pub(crate) id: u32,
    pub(crate) frame_id: u32,
    pub(crate) flags: PointerFlags,
    pub(crate) location_px: Point<u32>,
}

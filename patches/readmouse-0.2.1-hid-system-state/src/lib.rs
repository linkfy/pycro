#![allow(unused)]
#![allow(dead_code)]
use std::os::raw::c_void;

type CFTypeRef = *const c_void;
type CGEventRef = *const c_void;
type CGEventSourceRef = *const c_void;
type CGMouseButton = Mouse;

#[repr(i32)]
enum CGEventSourceStateID {
    Private = -1,
    Combined = 0,
    System = 1,
}

#[derive(Clone, Copy)]
#[repr(C)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {
    fn CGEventCreate(source: CGEventSourceRef) -> CGEventRef;
    fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    fn CGEventSourceButtonState(stateID: CGEventSourceStateID, button: CGMouseButton) -> bool;
    fn CFRelease(cf: CFTypeRef);
}

#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Mouse {
    Left,
    Right,
    Center,
}

impl Mouse {
    /// Returns the current mouse location.
    /// ```
    /// use readmouse::Mouse;
    /// loop {
    ///   println!("(x,y) = {:?}", Mouse::location());
    /// }
    /// ```
    #[inline(always)]
    pub fn location() -> (f64, f64) {
        unsafe {
            let event = CGEventCreate(std::ptr::null());
            let CGPoint { x, y } = CGEventGetLocation(event);
            CFRelease(event);
            (x, y)
        }
    }

    /// Checks if mouse button is pressed.
    /// ```
    /// use readmouse::Mouse;
    /// loop {
    ///   println!("Left button pressed? {:?}", Mouse::Left.is_pressed());
    /// }
    /// ```
    #[inline(always)]
    pub fn is_pressed(self) -> bool {
        unsafe { CGEventSourceButtonState(CGEventSourceStateID::System, self) }
    }
}

//! Place the main window once, after eframe restores it and before its first paint.

use winit::window::Window;

#[cfg(not(target_os = "macos"))]
use winit::dpi::PhysicalPosition;

#[cfg(not(target_os = "macos"))]
pub(crate) fn on_pointer_monitor(window: &Window) {
    let Some(pointer) = pointer_position(window) else {
        // In particular, native Wayland does not expose global pointer coordinates
        // or allow clients to position top-level windows.
        return;
    };
    let Some(monitor) = window.available_monitors().find(|monitor| {
        let origin = monitor.position();
        let size = monitor.size();
        contains(
            (origin.x, origin.y),
            (size.width, size.height),
            (pointer.x, pointer.y),
        )
    }) else {
        return;
    };

    let origin = monitor.position();
    let size = monitor.size();
    // Preserve the saved logical size when moving between different DPI scales.
    let outer = window.outer_size().to_logical::<f64>(window.scale_factor());
    let outer = outer.to_physical::<u32>(monitor.scale_factor());
    window.set_outer_position(PhysicalPosition::new(
        centered_axis(origin.x as f64, size.width as f64, outer.width as f64) as i32,
        centered_axis(origin.y as f64, size.height as f64, outer.height as f64) as i32,
    ));
}

fn centered_axis(origin: f64, extent: f64, window_extent: f64) -> f64 {
    // Keep the title bar on the display even if the saved window is too large.
    origin + ((extent - window_extent) / 2.0).max(0.0)
}

#[cfg(any(not(target_os = "macos"), test))]
fn contains(origin: (i32, i32), size: (u32, u32), point: (i32, i32)) -> bool {
    let x = i64::from(point.0) - i64::from(origin.0);
    let y = i64::from(point.1) - i64::from(origin.1);
    x >= 0 && y >= 0 && x < i64::from(size.0) && y < i64::from(size.1)
}

#[cfg(target_os = "windows")]
fn pointer_position(_window: &Window) -> Option<PhysicalPosition<i32>> {
    #[repr(C)]
    struct Point {
        x: i32,
        y: i32,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetCursorPos(point: *mut Point) -> i32;
    }
    let mut point = Point { x: 0, y: 0 };
    // SAFETY: GetCursorPos receives a valid writable Win32 POINT. Winit has
    // already initialized DPI awareness, so its monitors use the same units.
    let success = unsafe { GetCursorPos(&mut point) };
    (success != 0).then_some(PhysicalPosition::new(point.x, point.y))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn pointer_position(window: &Window) -> Option<PhysicalPosition<i32>> {
    use winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
    use x11_dl::xlib;

    let handle = window.display_handle().ok()?;
    let RawDisplayHandle::Xlib(display) = handle.as_raw() else {
        return None;
    };
    let connection = display.display?.as_ptr().cast::<xlib::Display>();
    let xlib = xlib::Xlib::open().ok()?;
    let (mut root, mut child, mut mask) = (0, 0, 0);
    let (mut x, mut y, mut local_x, mut local_y) = (0, 0, 0, 0);
    // SAFETY: The live winit display owns this connection and screen. All output
    // pointers refer to initialized values of the types required by XQueryPointer.
    // This runs on the GUI thread and does not close the borrowed connection.
    let success = unsafe {
        let screen_root = (xlib.XRootWindow)(connection, display.screen);
        (xlib.XQueryPointer)(
            connection,
            screen_root,
            &mut root,
            &mut child,
            &mut x,
            &mut y,
            &mut local_x,
            &mut local_y,
            &mut mask,
        )
    };
    (success != 0).then_some(PhysicalPosition::new(x, y))
}

#[cfg(not(any(unix, target_os = "windows")))]
fn pointer_position(_window: &Window) -> Option<PhysicalPosition<i32>> {
    None
}

#[cfg(target_os = "macos")]
pub(crate) fn on_pointer_monitor(window: &Window) {
    use std::ffi::c_void;
    use winit::dpi::LogicalPosition;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Point {
        x: f64,
        y: f64,
    }
    #[repr(C)]
    struct Size {
        width: f64,
        height: f64,
    }
    #[repr(C)]
    struct Rect {
        origin: Point,
        size: Size,
    }
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventCreate(source: *const c_void) -> *const c_void;
        fn CGEventGetLocation(event: *const c_void) -> Point;
        fn CGGetDisplaysWithPoint(
            point: Point,
            capacity: u32,
            displays: *mut u32,
            count: *mut u32,
        ) -> i32;
        fn CGDisplayBounds(display: u32) -> Rect;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFRelease(value: *const c_void);
    }

    // SAFETY: A null source requests the current system state. Check the owned
    // event before reading it, release it once, and provide valid output buffers.
    let bounds = unsafe {
        let event = CGEventCreate(std::ptr::null());
        if event.is_null() {
            return;
        }
        let pointer = CGEventGetLocation(event);
        CFRelease(event);
        let (mut display, mut count) = (0, 0);
        if CGGetDisplaysWithPoint(pointer, 1, &mut display, &mut count) != 0 || count == 0 {
            return;
        }
        CGDisplayBounds(display)
    };
    // CoreGraphics desktop bounds and winit logical positions both use points
    // from the top-left of the main display, including negative monitor origins.
    let outer = window.outer_size().to_logical::<f64>(window.scale_factor());
    window.set_outer_position(LogicalPosition::new(
        centered_axis(bounds.origin.x, bounds.size.width, outer.width),
        centered_axis(bounds.origin.y, bounds.size.height, outer.height),
    ));
}

#[cfg(test)]
mod tests {
    use super::{centered_axis, contains};

    #[test]
    fn monitor_edges_belong_to_only_one_display() {
        assert!(contains((-1920, -1080), (1920, 1080), (-1, -1)));
        assert!(!contains((-1920, -1080), (1920, 1080), (0, -1)));
        assert!(!contains((-1920, -1080), (1920, 1080), (-1, 0)));
        assert!(contains((0, 0), (1920, 1080), (0, 0)));
        assert!(!contains((0, 0), (1920, 1080), (-1, 0)));
    }

    #[test]
    fn centers_on_offset_displays_and_keeps_oversized_title_bar_visible() {
        assert_eq!(centered_axis(-1920.0, 1920.0, 568.0), -1244.0);
        assert_eq!(centered_axis(-1080.0, 1080.0, 920.0), -1000.0);
        assert_eq!(centered_axis(1920.0, 2560.0, 1136.0), 2632.0);
        assert_eq!(centered_axis(-720.0, 720.0, 920.0), -720.0);
    }
}

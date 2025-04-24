use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};
use std::{ffi::c_void, ptr::NonNull};

pub struct LWJGLGLFWWindow {
    native_window: *mut c_void,
}

impl LWJGLGLFWWindow {
    pub unsafe fn new(native_window: *mut c_void) -> Self {
        Self { native_window }
    }
}

unsafe impl Send for LWJGLGLFWWindow {}
unsafe impl Sync for LWJGLGLFWWindow {}

impl HasWindowHandle for LWJGLGLFWWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Ok(unsafe { WindowHandle::borrow_raw(raw_window_handle(self.native_window)) })
    }
}

impl HasDisplayHandle for LWJGLGLFWWindow {
    fn display_handle(&'_ self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe { DisplayHandle::borrow_raw(raw_display_handle()) })
    }
}

//Credit to glfw crate for (some of) this

fn raw_window_handle(native_window: *mut c_void) -> RawWindowHandle {
    #[cfg(target_family = "windows")]
    {
        use raw_window_handle::Win32WindowHandle;
        let (hwnd, hinstance) = unsafe {
            let hinstance: isize = winapi::um::winuser::GetWindowLongPtrA(
                native_window as _,
                winapi::um::winuser::GWLP_HINSTANCE,
            );

            (native_window, hinstance)
        };
        let mut handle = Win32WindowHandle::new(NonZeroIsize::new(hwnd as isize).unwrap());
        handle.hinstance = NonZeroIsize::new(hinstance);
        RawWindowHandle::Win32(handle)
    }
    #[cfg(target_os = "linux")]
    {
        // TODO: minecraft's glfw doesn't create wayland windows, but using a native patched glfw
        // does not work either as it segfaults everywhere for some goddamn reason, so we are x11
        // only for now #sad
        use raw_window_handle::XlibWindowHandle;
        RawWindowHandle::Xlib(XlibWindowHandle::new(native_window as u64))
    }
    #[cfg(target_os = "macos")]
    {
        use objc2::msg_send;
        use objc2::rc::Retained;
        use objc2::runtime::NSObject;
        use raw_window_handle::AppKitWindowHandle;
        let ns_window = native_window as *mut NSObject;
        let ns_view: Option<Retained<NSObject>> = unsafe { msg_send![ns_window, contentView] };
        let ns_view = ns_view.expect("failed to access contentView on GLFW NSWindow");
        let ns_view: NonNull<NSObject> = NonNull::from(&*ns_view);
        let handle = AppKitWindowHandle::new(ns_view.cast());
        RawWindowHandle::AppKit(handle)
    }
}

fn raw_display_handle() -> RawDisplayHandle {
    #[cfg(target_family = "windows")]
    {
        use raw_window_handle::WindowsDisplayHandle;
        RawDisplayHandle::Windows(WindowsDisplayHandle::new())
    }
    #[cfg(target_os = "linux")]
    {
        use raw_window_handle::XlibDisplayHandle;
        use x11_dl::xlib;

        let xlib = xlib::Xlib::open().expect("Could not open Xlib");
        let display = NonNull::new(unsafe { (xlib.XOpenDisplay)(std::ptr::null()) as *mut c_void });
        let handle = XlibDisplayHandle::new(display, 0);
        RawDisplayHandle::Xlib(handle)
    }
    #[cfg(target_os = "macos")]
    {
        use raw_window_handle::AppKitDisplayHandle;
        RawDisplayHandle::AppKit(AppKitDisplayHandle::new())
    }
}

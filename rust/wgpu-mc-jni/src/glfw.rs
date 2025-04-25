use objc2::msg_send;
use objc2::rc::Retained;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};
use std::ffi::c_void;

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
    #[cfg(all(
        any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"),
        not(feature = "wayland")
    ))]
    {
        use raw_window_handle::XlibWindowHandle;

        RawWindowHandle::Xlib(XlibWindowHandle::new(native_window))
    }
    #[cfg(all(
        any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"),
        feature = "wayland"
    ))]
    {
        use std::ptr::NonNull;

        use raw_window_handle::WaylandWindowHandle;

        let handle = WaylandWindowHandle::new(
            NonNull::new(native_window).expect("wayland window surface is null"),
        );
        RawWindowHandle::Wayland(handle)
    }
    #[cfg(target_os = "macos")]
    {
        use std::ptr::NonNull;

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
    #[cfg(all(
        any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"),
        not(feature = "wayland")
    ))]
    {
        use std::ptr::NonNull;

        use raw_window_handle::XlibDisplayHandle;
        let display = NonNull::new(unsafe { ffi::glfwGetX11Display() });
        let handle = XlibDisplayHandle::new(display, 0);
        RawDisplayHandle::Xlib(handle)
    }
    #[cfg(all(
        any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly"),
        feature = "wayland"
    ))]
    {
        use std::ptr::NonNull;

        use raw_window_handle::WaylandDisplayHandle;
        let display =
            NonNull::new(unsafe { ffi::glfwGetWaylandDisplay() }).expect("wayland display is null");
        let handle = WaylandDisplayHandle::new(display);
        RawDisplayHandle::Wayland(handle)
    }
    #[cfg(target_os = "macos")]
    {
        use raw_window_handle::AppKitDisplayHandle;
        RawDisplayHandle::AppKit(AppKitDisplayHandle::new())
    }
}

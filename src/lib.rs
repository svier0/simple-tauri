extern crate self as simple_tauri;

pub mod config;
pub mod simple_serve;
pub mod simple_tray;
pub mod utils;

#[cfg(windows)]
pub fn check_mutex(name: &str) -> bool {
    let mutex_name: Vec<u16> = format!("{name}-single-instance")
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mutex = unsafe {
        windows_sys::Win32::System::Threading::CreateMutexW(
            std::ptr::null(),
            0,
            mutex_name.as_ptr(),
        )
    };
    if mutex.is_null() {
        eprintln!("创建互斥体失败");
        return false;
    }
    let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
    if err != windows_sys::Win32::Foundation::ERROR_ALREADY_EXISTS {
        return false;
    }
    let msg: Vec<u16> = format!("{name} 已在运行")
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let title: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
            std::ptr::null_mut(),
            msg.as_ptr(),
            title.as_ptr(),
            0,
        )
    };
    true
}
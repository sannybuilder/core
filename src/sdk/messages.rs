use winapi::um::winuser::PostMessageA;

pub const WM_ONSTATUSCHANGE: u32 = winapi::um::winuser::WM_USER + 1048;
pub const WM_ONSHOWTEXTBOX: u32 = winapi::um::winuser::WM_USER + 1049;
pub const WM_CHANGETITLE: u32 = winapi::um::winuser::WM_USER + 1050;
pub const WM_RESETTITLE: u32 = winapi::um::winuser::WM_USER + 1051;
pub const WM_OPENFILE: u32 = winapi::um::winuser::WM_USER + 1052;

pub fn send_message(message: u32, wparam: usize, lparam: isize) {
    unsafe {
        PostMessageA(*super::GET_WORKBOOK_HANDLE as _, message, wparam, lparam);
    }
}

use std::ptr::null_mut;

use winapi::{ctypes::c_int, shared::{minwindef::{WPARAM, LPARAM, LRESULT}, windef::HWND}, um::{winuser::{MSG, HC_ACTION}}};

static mut GLOB_HWND: HWND = null_mut();

#[no_mangle]
pub unsafe extern "system" fn set_hwnd(hwnd: HWND) {
    dbg!(GLOB_HWND);
    GLOB_HWND = hwnd;
    dbg!(GLOB_HWND);
}

#[no_mangle]
pub unsafe extern "system" fn get_msg_proc(code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    use winapi::um::winuser::{CallNextHookEx, WM_INPUT, WM_KEYDOWN, WM_SYSKEYDOWN};
    if code < 0 || code == HC_ACTION {
        return CallNextHookEx(null_mut(), code, w_param, l_param);
    }
    let msg: &MSG = &*(l_param as *mut MSG);
    println!("msg_hook: processing msg");
    match (msg.hwnd == GLOB_HWND, msg.message) {
        (false, WM_INPUT) => {
            println!("Blocked keypress");
            return 1;
        },
        (false, WM_KEYDOWN) => {
            println!("Blocked keypress");
            return 1;
        },
        (false, WM_SYSKEYDOWN) => {
            println!("Blocked keypress");
            return 1;
        },
        (false, _) => {return CallNextHookEx(null_mut(), code, w_param, l_param);},
        (true, _) => {return CallNextHookEx(null_mut(), code, w_param, l_param);},
    }
}
use std::ptr::null_mut;

use winapi::{ctypes::c_int, shared::{minwindef::{WPARAM, LPARAM, LRESULT, HINSTANCE}, windef::HWND}, um::{winuser::{MSG, HC_ACTION}, libloaderapi::{GetModuleHandleExW, GetModuleHandleW}}};

static mut GLOB_HWND: HWND = null_mut();

#[no_mangle]
pub unsafe extern "system" fn set_hwnd(hwnd: HWND) {
    GLOB_HWND = hwnd;
}

#[no_mangle]
pub unsafe extern "system" fn get_msg_proc(code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    use winapi::um::winuser::{CallNextHookEx, WM_INPUT, WM_KEYDOWN, WM_SYSKEYDOWN};
    match code {
        HC_ACTION => {
            let msg: &MSG = &*(l_param as *mut MSG);
            match (msg.hwnd == GLOB_HWND, msg.message) {
                (false, WM_INPUT) => {
                    println!("Blocked keypress");
                    return CallNextHookEx(null_mut(), 1, w_param, l_param);
                },
                (false, WM_KEYDOWN) => {
                    println!("Blocked keypress");
                    return CallNextHookEx(null_mut(), 1, w_param, l_param);
                },
                (false, WM_SYSKEYDOWN) => {
                    println!("Blocked keypress");
                    return CallNextHookEx(null_mut(), 1, w_param, l_param);
                },
                (false, _) => {return CallNextHookEx(null_mut(), code, w_param, l_param);},
                (true, _) => {return CallNextHookEx(null_mut(), code, w_param, l_param);},
            }
        },
        _ => return CallNextHookEx(null_mut(), code, w_param, l_param),
    }
}
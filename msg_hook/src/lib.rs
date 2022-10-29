#![allow(clippy::missing_safety_doc, clippy::needless_return)]

use std::ptr::null_mut;

use winapi::{ctypes::c_int, shared::{minwindef::{WPARAM, LPARAM, LRESULT, UINT}, windef::HWND}, um::{winuser::{HC_ACTION, SendMessageW, WM_USER}}};

const WM_SHOULDBLKKEY: UINT = WM_USER + 300;

static mut GLOB_HWND: HWND = null_mut();

#[no_mangle]
pub unsafe extern "system" fn set_hwnd(hwnd: HWND) {
    GLOB_HWND = hwnd;
    dbg!(("hook dll set window: ", GLOB_HWND));
}

#[no_mangle]
pub unsafe extern "system" fn key_hook_proc(code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    use winapi::um::winuser::{CallNextHookEx};
    if code < 0 || code != HC_ACTION { // normal hooks process when code == HC_ACTION, but we want to intercept any messages we see
        dbg!("Hook: no action");
        return CallNextHookEx(null_mut(), code, w_param, l_param);
    }
    let kill = SendMessageW(GLOB_HWND, WM_SHOULDBLKKEY, w_param, l_param);
    match kill {
        0 => {
            dbg!("Hook: no kill");
            return CallNextHookEx(null_mut(), code, w_param, l_param);
        },
        _ => {
            dbg!("Hook: kill");
            return 1;
        }
    }
}
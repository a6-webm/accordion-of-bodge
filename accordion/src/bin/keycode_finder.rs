#![allow(clippy::missing_safety_doc, clippy::needless_return)]

use std::mem::size_of;
use std::ptr::null_mut;
use winapi::shared::minwindef::{UINT, LRESULT, WPARAM, LPARAM, LPVOID};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::processthreadsapi::{GetStartupInfoW, STARTUPINFOW};
use winapi::um::winuser::{WNDCLASSEXW, RegisterClassExW, CreateWindowExW, WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, GetMessageW, DispatchMessageW, MSG, PostQuitMessage, RAWINPUTDEVICE, RIDEV_NOLEGACY, RegisterRawInputDevices, RIDEV_INPUTSINK, RAWINPUT, WM_INPUT, HRAWINPUT, RID_INPUT, RAWINPUTHEADER, GetRawInputData, RIM_TYPEKEYBOARD};

fn main() {
    let h_instance = unsafe { GetModuleHandleW(null_mut()) };

    let mut startup_info: STARTUPINFOW;
    unsafe {
        startup_info = std::mem::zeroed();
        GetStartupInfoW(&mut startup_info); // Don't think I need to free this
    };

    let class_name = win32_string("my_first_window");

    let wc = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: 0,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: null_mut(),
    };

    unsafe { RegisterClassExW(&wc); }

    let hwnd: HWND = unsafe { CreateWindowExW(
            0,
            class_name.as_ptr(),
            win32_string("the bruh window").as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            null_mut(),
            null_mut(),
            h_instance,
            null_mut(),
    )};
    if hwnd.is_null() { panic!("failed to create window"); }

    let rid_tlc = RAWINPUTDEVICE {
        usUsagePage: 1, // HID_USAGE_PAGE_GENERIC
        usUsage: 6, // HID_USAGE_GENERIC_KEYBOARD
        dwFlags: RIDEV_NOLEGACY | RIDEV_INPUTSINK, // ignores legacy keyboard messages and reads input when not focused
        hwndTarget: hwnd,
    };

    unsafe { 
        if RegisterRawInputDevices(&rid_tlc, 1, size_of::<RAWINPUTDEVICE>() as UINT) == 0 {
            panic!("failed to register raw input TLC");
        }
    }

    unsafe {
        let mut lp_msg: MSG = std::mem::zeroed();
        println!("Press Ctrl+c to quit");
        println!("vvv Press any key to see its virtual key code vvv");
        while GetMessageW(&mut lp_msg, 0 as HWND, 0, 0) > 0 {
            DispatchMessageW(&lp_msg);
        }
    }
}

unsafe extern "system" fn wnd_proc(h_wnd: HWND, i_message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    use winapi::{um::winuser::{DefWindowProcW, WM_DESTROY}};

    match i_message {
        WM_DESTROY => {
            dbg!(PostQuitMessage(0));
            return 0;
        },
        WM_INPUT => {
            let mut rid_size: UINT = 0;
            GetRawInputData(l_param as HRAWINPUT, RID_INPUT, null_mut(), &mut rid_size, size_of::<RAWINPUTHEADER>() as UINT);
            if rid_size == 0 { return 0; } // not sure if this can happen, but microsoft docs do this
            let mut raw_data_buffer: Vec<u8> = Vec::with_capacity(rid_size.try_into().unwrap());

            if rid_size != GetRawInputData(
                l_param as HRAWINPUT,
                RID_INPUT,
                raw_data_buffer.as_mut_ptr() as LPVOID,
                &mut rid_size,
                size_of::<RAWINPUTHEADER>() as UINT
            ) {
                println!("GetRawInputData does not return correct size!");
            }

            let raw: &RAWINPUT = &*(raw_data_buffer.as_ptr() as *const RAWINPUT);

            if raw.header.dwType == RIM_TYPEKEYBOARD {
                println!("Device: {:?} - Key: {}", raw.header.hDevice, raw.data.keyboard().VKey);
            }
            return 0;
        },
        _ => DefWindowProcW(h_wnd, i_message, w_param, l_param),
    }
}

fn win32_string( value : &str ) -> Vec<u16> {
    use std::{ffi::OsStr, os::windows::prelude::OsStrExt, iter::once};
    OsStr::new( value ).encode_wide().chain( once( 0 ) ).collect()
}
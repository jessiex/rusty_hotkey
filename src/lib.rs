extern crate winapi;
extern crate user32;

use winapi::{MSLLHOOKSTRUCT, MOUSEINPUT, POINT, HWND, MSG, INPUT, KEYBDINPUT, c_int, WPARAM, LPARAM, LRESULT, HINSTANCE, HHOOK, KBDLLHOOKSTRUCT};
use user32::{GetAsyncKeyState, UnhookWindowsHookEx, PostMessageA, GetMessageW, GetKeyState, MapVirtualKeyA, SendInput, SetWindowsHookExA, CallNextHookEx};
use std::mem::{transmute, size_of, uninitialized};
use Input::{MousePressMiddle, MouseReleaseMiddle, KeybdPress, KeybdRelease, MousePressLeft, MouseReleaseLeft, MousePressRight, MouseReleaseRight, MouseMove, MouseWheel};

/// VirtalKey codes can be obtained from https://msdn.microsoft.com/en-us/library/windows/desktop/dd375731(v=vs.85).aspx
pub type VirtualKeyCode = u8;

#[derive(PartialEq, Debug)]
pub enum Input {
    KeybdPress(VirtualKeyCode),
    KeybdRelease(VirtualKeyCode),
    MousePressLeft(i32, i32),
    MouseReleaseLeft(i32, i32),
    MousePressMiddle(i32, i32),
    MouseReleaseMiddle(i32, i32),
    MousePressRight(i32, i32),
    MouseReleaseRight(i32, i32),
    MouseMove(i32, i32),
    MouseWheel(i32, i32, i32)
}

static mut KEYBD_HHOOK: HHOOK = 0 as HHOOK;
static mut MOUSE_HHOOK: HHOOK = 0 as HHOOK;

unsafe extern "system" fn hhook_proc(code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    PostMessageA(0 as HWND, 0, w_param, l_param);
    CallNextHookEx(KEYBD_HHOOK, code, w_param, l_param)
}


pub fn send_input(input: Input) {
    let mut _input: INPUT = match &input {
        &KeybdPress(vk_code) | &KeybdRelease(vk_code) => INPUT{
            type_: 1, 
            u: unsafe{transmute((KEYBDINPUT{
                wVk: 0,
                wScan: MapVirtualKeyA(vk_code as u32, 0) as u16,
                dwFlags: if let KeybdPress(_) = input {0x0008} else {0x0008 | 0x0002},
                time: 0,
                dwExtraInfo: 0
            }, 0 as u32))}
        },
        &MousePressLeft(x, y) | 
        &MouseReleaseLeft(x, y) | 
        &MousePressMiddle(x, y) |
        &MouseReleaseMiddle(x, y) |
        &MousePressRight(x, y) |
        &MouseReleaseRight(x, y) | 
        &MouseMove(x, y) | 
        &MouseWheel(x, y, _) => INPUT{
            type_: 1, 
            u: unsafe{transmute(MOUSEINPUT{
                dx: x,
                dy: y,
                mouseData: 0,
                dwFlags: match &input {
                    &MousePressLeft(_, _) => 0x0002,
                    &MouseReleaseLeft(_, _) => 0x0004,
                    &MousePressMiddle(_, _) => 0x0020,
                    &MouseReleaseMiddle(_, _) => 0x0040,
                    &MousePressRight(_, _) => 0x0008,
                    &MouseReleaseRight(_, _) => 0x0010,
                    &MouseMove(_, _) => 0x0001,
                    &MouseWheel(_, _, _) => 0x0800,
                    _ => 0
                },
                time: 0,
                dwExtraInfo: 0
            })}
        }
    };
    unsafe{SendInput(1, &mut _input, size_of::<INPUT>() as i32)};
}

/// The function returns inputs in the same form as used by 'send_input'.
///
/// #Example
/// ```
/// while let Some(input) = ::intercept_input() {
///  match input {
///   //Exit if NumLock gets pressed
///   ::KeybdRelease(144) => break,
///   //Log all inputs
///   _ => println!("{:?}", input)
///  }
/// }
/// ```
pub fn intercept_input() -> Option<Input> {
    unsafe{KEYBD_HHOOK = SetWindowsHookExA(13, Some(hhook_proc), 0 as HINSTANCE, 0)};
    unsafe{MOUSE_HHOOK = SetWindowsHookExA(14, Some(hhook_proc), 0 as HINSTANCE, 0)};
    let mut msg: MSG = unsafe{uninitialized()};
    if unsafe {GetMessageW(&mut msg, 0 as HWND, 0, 0)} <= 0 {return None}
    unsafe{UnhookWindowsHookEx(KEYBD_HHOOK)};
    unsafe{UnhookWindowsHookEx(MOUSE_HHOOK)};
    //Code can be found here: https://wiki.winehq.org/List_Of_Windows_Messages
    match msg.wParam {
        256 => Some(Input::KeybdPress(unsafe{*(msg.lParam as *const KBDLLHOOKSTRUCT)}.vkCode as u8)),
        257 => Some(Input::KeybdRelease(unsafe{*(msg.lParam as *const KBDLLHOOKSTRUCT)}.vkCode as u8)),
        512 | 513 | 514 | 516 | 517 | 519 | 520 | 522 => {
            let x = unsafe{*(msg.lParam as *const MSLLHOOKSTRUCT)}.pt.x;
            let y = unsafe{*(msg.lParam as *const MSLLHOOKSTRUCT)}.pt.y;
            match msg.wParam {
                512 => Some(MouseMove(x, y)),
                513 => Some(MousePressLeft(x, y)),
                514 => Some(MouseReleaseLeft(x, y)),
                516 => Some(MousePressRight(x, y)),
                517 => Some(MouseReleaseRight(x, y)),
                519 => Some(MousePressMiddle(x, y)),
                520 => Some(MouseReleaseMiddle(x, y)),
                522 => Some(MouseWheel(x, y, 0)),
                _ => None
            }
        },
        _ => None
    }
}

pub fn get_toggle_state(vk_code: VirtualKeyCode) -> bool {
    unsafe {GetKeyState(vk_code as i32) & 15 != 0}
}

///Returns whether a key is being physically pressed down
pub fn get_async_state(vk_code: VirtualKeyCode) -> bool {
    unsafe {GetAsyncKeyState(vk_code as i32) & 15 != 0}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        while let Some(input) = ::intercept_input() {
            match input {
                //Exit if NumLock gets pressed
                ::KeybdRelease(144) => break,
                //Log all inputs
                _ => println!("{:?}", input)
            }
        }
    }
}

use std::convert::{TryFrom};

use super::*;
use crate::enumflags::{TEnumFlags32, SEnumFlags32};

use arrayvec::ArrayVec;
use bitflags::*;

use winapi::shared::hidusage::*;
use winapi::shared::ntdef::NULL;
use winapi::um::winuser::RAWMOUSE;

pub enum EUsagePage {
    Generic,
}

impl EUsagePage {
    pub fn wintype(&self) -> USAGE {

        match self {
            Self::Generic => HID_USAGE_PAGE_GENERIC,
        }
    }
}

pub enum EUsage {
    GenericMouse,
}

impl EUsage {
    pub fn wintype(&self) -> USAGE {

        match self {
            Self::GenericMouse => HID_USAGE_GENERIC_MOUSE,
        }
    }
}

// -- stands for Raw Input DEVice as far as I can tell
#[derive(Copy, Clone)]
pub enum ERIDEV {
    AppKeys,
    CaptureMouse,
    DevNotify,
    Exclude,
    ExInputSink,
    InputSink,
    NoHotKeys,
    NoLegacy,
    PageOnly,
    Remove,
}

impl TEnumFlags32 for ERIDEV {
    type TRawType = DWORD;

    fn rawtype(&self) -> Self::TRawType {
        use winapi::um::winuser::*;

        match self {
            Self::AppKeys => RIDEV_APPKEYS,
            Self::CaptureMouse => RIDEV_CAPTUREMOUSE,
            Self::DevNotify => RIDEV_DEVNOTIFY,
            Self::Exclude => RIDEV_EXCLUDE,
            Self::ExInputSink => RIDEV_EXINPUTSINK,
            Self::InputSink => RIDEV_INPUTSINK,
            Self::NoHotKeys => RIDEV_NOHOTKEYS,
            Self::NoLegacy => RIDEV_NOLEGACY,
            Self::PageOnly => RIDEV_PAGEONLY,
            Self::Remove => RIDEV_REMOVE,
        }
    }
}

pub type SRIDEV = SEnumFlags32<ERIDEV>;

pub struct SRawInputDevice<'a> {
    pub usage_page: EUsagePage,
    pub usage: EUsage,
    pub flags: SRIDEV,
    pub target: Option<&'a SWindow>,
}

impl<'a> SRawInputDevice<'a> {
    pub unsafe fn wintype(&self) -> RAWINPUTDEVICE {
        RAWINPUTDEVICE {
            usUsagePage: self.usage_page.wintype(),
            usUsage: self.usage.wintype(),
            dwFlags: self.flags.rawtype(),
            hwndTarget: match self.target {
                None => NULL as HWND,
                Some(window) => window.raw(),
            },
        }
    }
}

pub fn register_raw_input_devices(raw_input_devices: &[SRawInputDevice]) -> Result<(), &'static str> {
    assert!(raw_input_devices.len() <= 4);
    let mut temp : ArrayVec<[RAWINPUTDEVICE; 4]>= ArrayVec::new();

    unsafe {
        for device in raw_input_devices {
            temp.push(device.wintype());
        }

        if temp.len() > 0 {
            let result = winapi::um::winuser::RegisterRawInputDevices(
                temp.as_mut_ptr(),
                temp.len() as u32,
                std::mem::size_of_val(&temp[0]) as u32,
            );

            if result == TRUE {
                return Ok(());
            }
            else {
                let _err = winapi::um::errhandlingapi::GetLastError();
                return Err("failed to register input devices.");
            }
        }

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub enum ERIMType {
    Mouse,
    Keyboard,
    HID,
}

impl ERIMType {
    pub fn wintype(&self) -> DWORD {
        match self {
            Self::Mouse => RIM_TYPEMOUSE,
            Self::Keyboard => RIM_TYPEKEYBOARD,
            Self::HID => RIM_TYPEHID,
        }
    }
}

impl TryFrom<DWORD> for ERIMType {
    type Error = &'static str;

    fn try_from(value: DWORD) -> Result<Self, Self::Error> {
        match value {
            RIM_TYPEMOUSE => Ok(Self::Mouse),
            RIM_TYPEKEYBOARD => Ok(Self::Keyboard),
            RIM_TYPEHID => Ok(Self::HID),
            _ => Err("invalid RIM_TYPE")
        }
    }
}

#[derive(Copy, Clone)]
pub struct SRawInputHeader {
    type_: ERIMType,
    size: usize,
    //handle: SDeviceHandle,
    //wparam: ???,
}

impl TryFrom<RAWINPUTHEADER> for SRawInputHeader {
    type Error = &'static str;

    fn try_from(value: RAWINPUTHEADER) -> Result<Self, Self::Error> {
        Ok(
            SRawInputHeader {
                type_: ERIMType::try_from(value.dwType)?,
                size: value.dwSize as usize,
            }
        )
    }
}

bitflags! {
    pub struct SRawMouseFlags: USHORT {
        const ATTRIBUTES_CHANGED = MOUSE_ATTRIBUTES_CHANGED;
        const MOVE_RELATIVE = MOUSE_MOVE_RELATIVE;
        const MOVE_ABSOLUTE = MOUSE_MOVE_ABSOLUTE;
        const VIRTUAL_DESKTOP = MOUSE_VIRTUAL_DESKTOP;
    }
}

bitflags! {
    pub struct SRIMouseButtonFlags: USHORT {
        const LEFT_BUTTON_DOWN = RI_MOUSE_LEFT_BUTTON_DOWN;
        const LEFT_BUTTON_UP = RI_MOUSE_LEFT_BUTTON_UP;
        const MIDDLE_BUTTON_DOWN = RI_MOUSE_MIDDLE_BUTTON_DOWN;
        const MIDDLE_BUTTON_UP = RI_MOUSE_MIDDLE_BUTTON_UP;
        const RIGHT_BUTTON_DOWN = RI_MOUSE_RIGHT_BUTTON_DOWN;
        const RIGHT_BUTTON_UP = RI_MOUSE_RIGHT_BUTTON_UP;
        const BUTTON_1_DOWN = RI_MOUSE_BUTTON_1_DOWN;
        const BUTTON_1_UP = RI_MOUSE_BUTTON_1_UP;
        const BUTTON_2_DOWN = RI_MOUSE_BUTTON_2_DOWN;
        const BUTTON_2_UP = RI_MOUSE_BUTTON_2_UP;
        const BUTTON_3_DOWN = RI_MOUSE_BUTTON_3_DOWN;
        const BUTTON_3_UP = RI_MOUSE_BUTTON_3_UP;
        const BUTTON_4_DOWN = RI_MOUSE_BUTTON_4_DOWN;
        const BUTTON_4_UP = RI_MOUSE_BUTTON_4_UP;
        const BUTTON_5_DOWN = RI_MOUSE_BUTTON_5_DOWN;
        const BUTTON_5_UP = RI_MOUSE_BUTTON_5_UP;
        const MOUSE_WHEEL = RI_MOUSE_WHEEL;
    }
}

#[derive(Copy, Clone)]
pub struct SRawMouse {
    pub flags: SRawMouseFlags,
    pub button_flags: SRIMouseButtonFlags,
    //u32: raw_buttons,
    pub last_x: i32,
    pub last_y: i32,
}

impl TryFrom<&RAWMOUSE> for SRawMouse {
    type Error = &'static str;

    fn try_from(value: &RAWMOUSE) -> Result<Self, Self::Error> {
        Ok(Self {
            flags: SRawMouseFlags::from_bits(value.usFlags).ok_or("Invalid flag bits.")?,
            button_flags: SRIMouseButtonFlags::from_bits(value.usButtonFlags).ok_or("Invalid button flag bits.")?,
            last_x: value.lLastX,
            last_y: value.lLastY,
        })
    }
}

#[derive(Copy, Clone)]
pub enum ERawInputData {
    Invalid,
    Mouse{ data: SRawMouse },
}

#[derive(Copy, Clone)]
pub struct SRawInput {
    pub header: SRawInputHeader,
    pub data: ERawInputData,
}

impl TryFrom<RAWINPUT> for SRawInput {
    type Error = &'static str;

    fn try_from(value: RAWINPUT) -> Result<Self, Self::Error> {
        let header = SRawInputHeader::try_from(value.header)?;
        let header_type = header.type_;
        Ok(Self {
            header: header,
            data: match header_type {
                ERIMType::Mouse => ERawInputData::Mouse {
                    data: SRawMouse::try_from(unsafe { value.data.mouse() })?,
                },
                ERIMType::Keyboard => ERawInputData::Invalid,
                ERIMType::HID => ERawInputData::Invalid,
            }
        })
    }
}

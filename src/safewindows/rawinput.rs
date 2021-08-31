use std::convert::{TryFrom};

use super::*;
use enumflags::{TEnumFlags, SEnumFlags};

use arrayvec::ArrayVec;
use bitflags::*;

pub enum EUsagePage {
    Generic,
}

pub type USAGE = u16;

impl EUsagePage {
    pub fn wintype(&self) -> USAGE {

        match self {
            Self::Generic => win::HID_USAGE_PAGE_GENERIC,
        }
    }
}

pub enum EUsage {
    GenericMouse,
}

impl EUsage {
    pub fn wintype(&self) -> USAGE {

        match self {
            Self::GenericMouse => win::HID_USAGE_GENERIC_MOUSE,
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

impl TEnumFlags for ERIDEV {
    type TRawType = win::RAWINPUTDEVICE_FLAGS;

    fn rawtype(&self) -> Self::TRawType {
        match self {
            Self::AppKeys => win::RIDEV_APPKEYS,
            Self::CaptureMouse => win::RIDEV_CAPTUREMOUSE,
            Self::DevNotify => win::RIDEV_DEVNOTIFY,
            Self::Exclude => win::RIDEV_EXCLUDE,
            Self::ExInputSink => win::RIDEV_EXINPUTSINK,
            Self::InputSink => win::RIDEV_INPUTSINK,
            Self::NoHotKeys => win::RIDEV_NOHOTKEYS,
            Self::NoLegacy => win::RIDEV_NOLEGACY,
            Self::PageOnly => win::RIDEV_PAGEONLY,
            Self::Remove => win::RIDEV_REMOVE,
        }
    }
}

pub type SRIDEV = SEnumFlags<ERIDEV>;

pub struct SRawInputDevice<'a> {
    pub usage_page: EUsagePage,
    pub usage: EUsage,
    pub flags: SRIDEV,
    pub target: Option<&'a SWindow>,
}

impl<'a> SRawInputDevice<'a> {
    pub unsafe fn wintype(&self) -> win::RAWINPUTDEVICE {
        win::RAWINPUTDEVICE {
            usUsagePage: self.usage_page.wintype(),
            usUsage: self.usage.wintype(),
            dwFlags: self.flags.rawtype(),
            hwndTarget: match self.target {
                None => win::HWND::NULL,
                Some(window) => window.raw(),
            },
        }
    }
}

pub fn register_raw_input_devices(raw_input_devices: &[SRawInputDevice]) -> Result<(), &'static str> {
    assert!(raw_input_devices.len() <= 4);
    let mut temp : ArrayVec<[win::RAWINPUTDEVICE; 4]>= ArrayVec::new();

    unsafe {
        for device in raw_input_devices {
            temp.push(device.wintype());
        }

        if temp.len() > 0 {
            let result = win::RegisterRawInputDevices(
                temp.as_mut_ptr(),
                temp.len() as u32,
                std::mem::size_of_val(&temp[0]) as u32,
            );

            if result == true {
                return Ok(());
            }
            else {
                let _err = win::GetLastError();
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
    pub fn wintype(&self) -> win::RID_DEVICE_INFO_TYPE {
        match self {
            Self::Mouse => win::RIM_TYPEMOUSE,
            Self::Keyboard => win::RIM_TYPEKEYBOARD,
            Self::HID => win::RIM_TYPEHID,
        }
    }
}

impl TryFrom<win::RID_DEVICE_INFO_TYPE> for ERIMType {
    type Error = &'static str;

    fn try_from(value: win::RID_DEVICE_INFO_TYPE) -> Result<Self, Self::Error> {
        match value {
            win::RIM_TYPEMOUSE => Ok(Self::Mouse),
            win::RIM_TYPEKEYBOARD => Ok(Self::Keyboard),
            win::RIM_TYPEHID => Ok(Self::HID),
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

impl TryFrom<win::RAWINPUTHEADER> for SRawInputHeader {
    type Error = &'static str;

    fn try_from(value: win::RAWINPUTHEADER) -> Result<Self, Self::Error> {
        Ok(
            SRawInputHeader {
                type_: ERIMType::try_from(win::RID_DEVICE_INFO_TYPE::from(value.dwType))?,
                size: value.dwSize as usize,
            }
        )
    }
}

bitflags! {
    pub struct SRawMouseFlags: u32 {
        const ATTRIBUTES_CHANGED = win::MOUSE_ATTRIBUTES_CHANGED;
        const MOVE_RELATIVE = win::MOUSE_MOVE_RELATIVE;
        const MOVE_ABSOLUTE = win::MOUSE_MOVE_ABSOLUTE;
        const VIRTUAL_DESKTOP = win::MOUSE_VIRTUAL_DESKTOP;
    }
}

bitflags! {
    pub struct SRIMouseButtonFlags: u32 {
        const LEFT_BUTTON_DOWN = win::RI_MOUSE_LEFT_BUTTON_DOWN;
        const LEFT_BUTTON_UP = win::RI_MOUSE_LEFT_BUTTON_UP;
        const MIDDLE_BUTTON_DOWN = win::RI_MOUSE_MIDDLE_BUTTON_DOWN;
        const MIDDLE_BUTTON_UP = win::RI_MOUSE_MIDDLE_BUTTON_UP;
        const RIGHT_BUTTON_DOWN = win::RI_MOUSE_RIGHT_BUTTON_DOWN;
        const RIGHT_BUTTON_UP = win::RI_MOUSE_RIGHT_BUTTON_UP;
        /*
        const BUTTON_1_DOWN = win::RI_MOUSE_BUTTON_1_DOWN;
        const BUTTON_1_UP = win::RI_MOUSE_BUTTON_1_UP;
        const BUTTON_2_DOWN = win::RI_MOUSE_BUTTON_2_DOWN;
        const BUTTON_2_UP = win::RI_MOUSE_BUTTON_2_UP;
        const BUTTON_3_DOWN = win::RI_MOUSE_BUTTON_3_DOWN;
        const BUTTON_3_UP = win::RI_MOUSE_BUTTON_3_UP;
        */
        const BUTTON_4_DOWN = win::RI_MOUSE_BUTTON_4_DOWN;
        const BUTTON_4_UP = win::RI_MOUSE_BUTTON_4_UP;
        const BUTTON_5_DOWN = win::RI_MOUSE_BUTTON_5_DOWN;
        const BUTTON_5_UP = win::RI_MOUSE_BUTTON_5_UP;
        const MOUSE_WHEEL = win::RI_MOUSE_WHEEL;
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

impl TryFrom<&win::RAWMOUSE> for SRawMouse {
    type Error = &'static str;

    fn try_from(value: &win::RAWMOUSE) -> Result<Self, Self::Error> {
        Ok(Self {
            flags: SRawMouseFlags::from_bits(value.usFlags as u32).ok_or("Invalid flag bits.")?,
            button_flags: SRIMouseButtonFlags::from_bits(unsafe { value.Anonymous.Anonymous.usButtonFlags as u32 }).ok_or("Invalid button flag bits.")?,
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

impl TryFrom<win::RAWINPUT> for SRawInput {
    type Error = &'static str;

    fn try_from(value: win::RAWINPUT) -> Result<Self, Self::Error> {
        let header = SRawInputHeader::try_from(value.header)?;
        let header_type = header.type_;
        Ok(Self {
            header: header,
            data: match header_type {
                ERIMType::Mouse => ERawInputData::Mouse {
                    data: SRawMouse::try_from(unsafe { &value.data.mouse })?,
                },
                ERIMType::Keyboard => ERawInputData::Invalid,
                ERIMType::HID => ERawInputData::Invalid,
            }
        })
    }
}

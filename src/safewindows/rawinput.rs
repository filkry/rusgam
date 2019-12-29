use super::*;
use enumflags::{TEnumFlags32, SEnumFlags32};

use arrayvec::ArrayVec;

use winapi::shared::hidusage::*;
use winapi::shared::ntdef::NULL;

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

pub fn register_raw_input_devices(raw_input_devices: &[SRawInputDevice]) {
    assert!(raw_input_devices.len() <= 4);
    let mut temp : ArrayVec<[RAWINPUTDEVICE; 4]>= ArrayVec::new();

    unsafe {
        for device in raw_input_devices {
            temp.push(device.wintype());
        }

        if temp.len() > 0 {
            winapi::um::winuser::RegisterRawInputDevices(
                temp.as_mut_ptr(),
                temp.len() as u32,
                std::mem::size_of_val(&temp[0]) as u32,
            );
        }
    }
}

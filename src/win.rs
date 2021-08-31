#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// -- sanitized collection of windows imports

pub use windows::Abi;
pub use windows::Interface;
pub use windows::IUnknown;

pub use winbindings::bindings::Windows::Win32::Devices::HumanInterfaceDevice::*;
pub use winbindings::bindings::Windows::Win32::Foundation::*;
pub use winbindings::bindings::Windows::Win32::Graphics::Direct3D11::{
    D3D_FEATURE_LEVEL_11_0,
    D3D_PRIMITIVE_TOPOLOGY,
    D3D_PRIMITIVE_TOPOLOGY_UNDEFINED,
    D3D_PRIMITIVE_TOPOLOGY_POINTLIST,
    D3D_PRIMITIVE_TOPOLOGY_LINELIST,
    D3D_PRIMITIVE_TOPOLOGY_LINESTRIP,
    D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
    D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
    D3D_PRIMITIVE_TOPOLOGY_LINELIST_ADJ,
    D3D_PRIMITIVE_TOPOLOGY_LINESTRIP_ADJ,
    D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST_ADJ,
    D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP_ADJ,
    ID3DBlob,
    ID3DInclude,
};
pub use winbindings::bindings::Windows::Win32::Graphics::Direct3D12::*;
pub use winbindings::bindings::Windows::Win32::Graphics::Dxgi::*;
pub use winbindings::bindings::Windows::Win32::Graphics::Gdi::{
    PAINTSTRUCT,
    BeginPaint,
    EndPaint,
    HBRUSH,
    ScreenToClient,
};
pub use winbindings::bindings::Windows::Win32::Graphics::Hlsl::*;
pub use winbindings::bindings::Windows::Win32::System::Diagnostics::Debug::{
    WIN32_ERROR,
    DebugBreak,
    GetLastError,
    IsDebuggerPresent
};
pub use winbindings::bindings::Windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetModuleHandleA};
pub use winbindings::bindings::Windows::Win32::System::Performance::*;
pub use winbindings::bindings::Windows::Win32::System::Threading::*;
pub use winbindings::bindings::Windows::Win32::UI::KeyboardAndMouseInput::*;
pub use winbindings::bindings::Windows::Win32::UI::WindowsAndMessaging::*;

// -- extra types from windows API that windows-rs doesn't expose for some reason
pub type D3D12_GPU_VIRTUAL_ADDRESS = u64;
pub type D3D12_RECT = RECT;

pub fn GET_X_LPARAM(lparam: LPARAM) -> i32 {
    lparam.0 as i32
}

pub fn GET_Y_LPARAM(lparam: LPARAM) -> i32 {
    (lparam.0 >> 32) as i32
}



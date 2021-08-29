// -- sanitized collection of windows imports

#![allow(non_camel_case_types)]

pub use winbindings2::Windows::Win32::Devices::HumanInterfaceDevice::*;
pub use winbindings2::Windows::Win32::Foundation::*;
pub use winbindings2::Windows::Win32::Graphics::Direct3D11::{
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
};
pub use winbindings2::Windows::Win32::Graphics::Direct3D12::*;
pub use winbindings2::Windows::Win32::Graphics::Dxgi::*;
pub use winbindings2::Windows::Win32::Graphics::Gdi::{
    PAINTSTRUCT,
    BeginPaint,
    EndPaint,
    HBRUSH,
    ScreenToClient,
};
pub use winbindings2::Windows::Win32::Graphics::Hlsl::*;
pub use winbindings2::Windows::Win32::System::Diagnostics::Debug::{
    WIN32_ERROR,
    DebugBreak,
    GetLastError,
    IsDebuggerPresent
};
pub use winbindings2::Windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetModuleHandleA};
pub use winbindings2::Windows::Win32::System::Performance::*;
pub use winbindings2::Windows::Win32::System::Threading::*;
pub use winbindings2::Windows::Win32::UI::KeyboardAndMouseInput::*;
pub use winbindings2::Windows::Win32::UI::WindowsAndMessaging::*;

// -- extra types from windows API that windows-rs doesn't expose for some reason

#[repr(transparent)]
pub struct D3D12_GPU_VIRTUAL_ADDRESS(u64);

#[repr(transparent)]
pub struct D3D12_RECT(RECT);

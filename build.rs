fn main() {
    windows::build! {
        Windows::Win32::Foundation::*,
        Windows::Win32::Graphics::Direct3D12::*,
        Windows::Win32::Graphics::Dxgi::*,
        Windows::Win32::Graphics::Gdi::*,
        Windows::Win32::System::Diagnostics::Debug::*,
        Windows::Win32::System::LibraryLoader::*,
        Windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency},
        Windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject},
        Windows::Win32::UI::WindowsAndMessaging::*,
    };
}
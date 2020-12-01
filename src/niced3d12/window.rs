use std::ops::{Deref, DerefMut};

use rustywindows;
use safewindows;
use typeyd3d12 as t12;

pub struct SD3D12Window {
    window: rustywindows::SWindow,
    pub swapchain: super::SSwapChain,

    curbuffer: usize,
    rtvdescriptorheap: super::SDescriptorHeap,
    curwidth: u32,
    curheight: u32,
}

impl Deref for SD3D12Window {
    type Target = rustywindows::SWindow;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl DerefMut for SD3D12Window {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.window
    }
}

impl SD3D12Window {
    pub fn new(
        windowclass: &safewindows::SWindowClass,
        factory: &super::SFactory,
        device: &super::SDevice,
        commandqueue: &super::SCommandQueue,
        title: &str,
        width: u32,
        height: u32,
    ) -> Result<Self, &'static str> {
        let window = rustywindows::SWindow::create(windowclass, title, width, height).unwrap(); // $$$FRK(TODO): this panics, need to unify error handling

        let swap_chain = factory.create_swap_chain(&window.raw(), commandqueue, width, height)?;
        let cur_buffer = swap_chain.current_backbuffer_index();

        let desc = t12::SDescriptorHeapDesc {
            type_: t12::EDescriptorHeapType::RenderTarget,
            num_descriptors: 10,
            flags: t12::SDescriptorHeapFlags::from(t12::EDescriptorHeapFlags::None),
        };

        let descriptor_heap = device.create_descriptor_heap(&desc)?;

        // -- set up raw mouse input events
        {
            use safewindows::rawinput::*;

            let dev = SRawInputDevice {
                usage_page: EUsagePage::Generic,
                usage: EUsage::GenericMouse,
                flags: SRIDEV::from(ERIDEV::InputSink),
                target: Some(window.raw()),
            };

            register_raw_input_devices(&[dev])?;
        }

        Ok(Self {
            window: window,
            swapchain: swap_chain,
            curbuffer: cur_buffer,
            rtvdescriptorheap: descriptor_heap,
            curwidth: width,
            curheight: height,
        })
    }

    pub fn init_render_target_views(
        &mut self,
        device: &super::SDevice,
    ) -> Result<(), &'static str> {
        device.init_render_target_views(&mut self.swapchain, &mut self.rtvdescriptorheap)?;
        Ok(())
    }

    // -- $$$FRK(TODO): need to think about this, non-mut seems wrong (as does just handing out a pointer in general)
    pub fn currentbackbuffer(&self) -> &super::SResource {
        &self.swapchain.backbuffers[self.curbuffer]
    }

    pub fn currentbackbufferindex(&self) -> usize {
        self.curbuffer
    }

    pub fn currentrendertargetdescriptor(&self) -> Result<t12::SCPUDescriptorHandle, &'static str> {
        self.rtvdescriptorheap.cpu_handle(self.curbuffer)
    }

    pub fn present(&mut self) -> Result<(), &'static str> {
        // -- $$$FRK(TODO): figure out what this value does
        let syncinterval = 1;
        self.swapchain.present(syncinterval, 0)?;
        let newbuffer = self.swapchain.current_backbuffer_index();
        assert!(newbuffer != self.curbuffer);
        self.curbuffer = newbuffer;

        Ok(())
    }

    pub fn width(&self) -> u32 {
        self.curwidth
    }

    pub fn height(&self) -> u32 {
        self.curheight
    }

    pub fn screen_pos_to_window(&self, screen_pos: [u32; 2]) -> [i32; 2] {
        let window_pos = self.window.screen_to_client(&screen_pos);

        window_pos
    }

    pub fn mouse_pos(&self, winapi: &safewindows::SWinAPI) -> [i32; 2] {
        let screen_cursor_pos = winapi.get_cursor_pos();
        self.screen_pos_to_window(screen_cursor_pos)
    }

    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
        commandqueue: &mut super::SCommandQueue,
        device: &super::SDevice,
    ) -> Result<(), &'static str> {
        if self.curwidth != width || self.curheight != height {
            let newwidth = std::cmp::max(1, width);
            let newheight = std::cmp::max(1, height);
            commandqueue.flush_blocking()?;

            self.swapchain.backbuffers.clear();

            let desc = self.swapchain.get_desc()?;
            self.swapchain
                .resize_buffers(2, newwidth, newheight, &desc)?;

            self.curbuffer = self.swapchain.current_backbuffer_index();
            self.init_render_target_views(device)?;

            self.curwidth = newwidth;
            self.curheight = newheight;
        }

        Ok(())
    }
}

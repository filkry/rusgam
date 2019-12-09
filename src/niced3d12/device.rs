use super::*;

pub struct SDevice {
    raw: t12::SDevice,
}

impl SDevice {
    pub fn new_from_raw(raw: t12::SDevice) -> Self {
        Self { raw: raw }
    }

    pub fn create_command_queue(
        &self,
        winapi: &safewindows::SWinAPI,
        type_: t12::ECommandListType,
    ) -> Result<SCommandQueue, &'static str> {
        let qresult = self.raw.createcommandqueue(type_)?;

        Ok(SCommandQueue::new_from_raw(
            qresult,
            self.create_fence(winapi)?,
            type_,
        ))
    }

    pub fn create_command_allocator(
        &self,
        type_: t12::ECommandListType,
    ) -> Result<SCommandAllocator, &'static str> {
        let raw = self.raw.createcommandallocator(type_)?;
        Ok(unsafe { SCommandAllocator::new_from_raw(raw) })
    }

    // -- NOTE: This is unsafe because it initializes the list to an allocator which we don't
    // -- know is exclusive to the list
    pub unsafe fn create_command_list(
        &self,
        allocator: &mut SCommandAllocator,
    ) -> Result<SCommandList, &'static str> {
        let raw = self.raw.createcommandlist(&allocator.raw())?;
        Ok(SCommandList::new_from_raw(raw))
    }

    pub fn create_fence(&self, winapi: &safewindows::SWinAPI) -> Result<SFence, &'static str> {
        let fence = self.raw.createfence()?;
        Ok(SFence::new_from_raw(
            fence,
            winapi.createeventhandle().unwrap(),
        ))
    }

    pub fn create_render_target_view(
        &self,
        render_target_resource: &mut SResource,
        dest_descriptor: &t12::SCPUDescriptorHandle,
    ) -> Result<(), &'static str> {
        // -- $$$FRK(TODO): assert on resource metadata
        self.raw
            .createrendertargetview(&render_target_resource.raw, dest_descriptor);
        Ok(())
    }

    pub fn create_depth_stencil_view(
        &self,
        depth_texture_resource: &mut SResource,
        desc: &t12::SDepthStencilViewDesc,
        dest_descriptor: t12::SCPUDescriptorHandle,
    ) -> Result<(), &'static str> {
        // -- $$$FRK(TODO): assert on resource metadata
        self.raw
            .create_depth_stencil_view(&depth_texture_resource.raw, desc, dest_descriptor);
        Ok(())
    }

    pub fn create_descriptor_heap(
        &self,
        type_: t12::EDescriptorHeapType,
        numdescriptors: u32,
    ) -> Result<SDescriptorHeap, &'static str> {
        //let raw = self.d.createdescriptorheap(type_, numdescriptors)?;

        let dh = self.raw.create_descriptor_heap(type_, numdescriptors)?;

        Ok(SDescriptorHeap {
            raw: dh,
            numdescriptors: numdescriptors,
            descriptorsize: self.raw.getdescriptorhandleincrementsize(type_),
            //cpudescriptorhandleforstart: raw.getcpudescriptorhandleforheapstart(),
        })
    }

    pub fn create_committed_texture2d_resource(
        &self, // verified thread safe via docs
        heap_type: t12::EHeapType,
        width: u32,
        height: u32,
        array_size: u16,
        mip_levels: u16,
        format: t12::EDXGIFormat,
        clear_value: t12::SClearValue,
        flags: t12::SResourceFlags,
        initial_resource_state: t12::EResourceStates,
    ) -> Result<SResource, &'static str> {
        let destinationresource = self.raw.createcommittedresource(
            t12::SHeapProperties::create(heap_type),
            t12::EHeapFlags::ENone,
            t12::SResourceDesc::create_texture_2d(
                width, height, array_size, mip_levels, format, flags,
            ),
            initial_resource_state,
            Some(clear_value),
        )?;

        Ok(SResource {
            raw: destinationresource,
            metadata: EResourceMetadata::Texture2DResource,
        })
    }

    pub fn create_committed_buffer_resource(
        &self, // verified thread safe via docs
        heap_type: t12::EHeapType,
        heap_flags: t12::EHeapFlags,
        flags: t12::SResourceFlags,
        initial_resource_state: t12::EResourceStates,
        num_items: usize,
        size_of_item: usize,
    ) -> Result<SResource, &'static str> {
        let destinationresource = self.raw.createcommittedresource(
            t12::SHeapProperties::create(heap_type),
            heap_flags,
            t12::SResourceDesc::createbuffer(num_items * size_of_item, flags),
            initial_resource_state,
            None,
        )?;

        Ok(SResource {
            raw: destinationresource,
            metadata: EResourceMetadata::BufferResource {
                count: num_items,
                sizeofentry: size_of_item,
            },
        })
    }

    pub fn create_committed_buffer_resource_for_data<T>(
        &self, // verified thread safe via docs
        heaptype: t12::EHeapType,
        flags: t12::SResourceFlags,
        initial_resource_state: t12::EResourceStates,
        bufferdata: &[T],
    ) -> Result<SResource, &'static str> {
        self.create_committed_buffer_resource(
            heaptype,
            t12::EHeapFlags::ENone,
            flags,
            initial_resource_state,
            bufferdata.len(),
            std::mem::size_of::<T>(),
        )
    }

    pub fn raw(&self) -> &t12::SDevice {
        &self.raw
    }

    pub fn raw_mut(&mut self) -> &mut t12::SDevice {
        &mut self.raw
    }

    pub fn init_render_target_views(
        &mut self,
        swap_chain: &mut SSwapChain,
        descriptor_heap: &mut SDescriptorHeap,
    ) -> Result<(), &'static str> {
        assert!(swap_chain.backbuffers.is_empty());

        match descriptor_heap.type_() {
            t12::EDescriptorHeapType::RenderTarget => {
                for backbuffidx in 0usize..2usize {
                    let rawresource = swap_chain.raw().getbuffer(backbuffidx)?;

                    let resource = SResource {
                        raw: rawresource,
                        metadata: EResourceMetadata::SwapChainResource,
                    };

                    swap_chain.backbuffers.push(resource);

                    let curdescriptorhandle = descriptor_heap.cpu_handle(backbuffidx)?;
                    self.create_render_target_view(
                        &mut swap_chain.backbuffers[backbuffidx],
                        &curdescriptorhandle,
                    )?;
                }

                Ok(())
            }
            _ => Err("Tried to initialize render target views on non-RTV descriptor heap."),
        }
    }
}

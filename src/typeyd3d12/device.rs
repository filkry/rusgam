use super::*;

#[derive(Clone)]
pub struct SDevice {
    device: win::ID3D12Device2,
}

pub fn d3d12createdevice(adapter: win::IUnknown) -> Result<SDevice, &'static str> {
    let hn = unsafe {
        win::D3D12CreateDevice(
            adapter,
            win::D3D_FEATURE_LEVEL_11_0,
        )
    };
    returnerrifwinerror!(hn, "Could not create device on adapter.");

    let device = hn.expect("checked for error above");
    Ok(SDevice { device: device })
}

impl SDevice {
    pub fn castinfoqueue(&self) -> Option<SInfoQueue> {
        use win::Interface;
        match self.device.cast::<win::ID3D12InfoQueue>() {
            Ok(a) => {
                return Some(unsafe { SInfoQueue::new_from_raw(a) });
            }
            Err(_) => {
                return None;
            }
        };
    }

    pub fn createcommandqueue(
        &self,
        type_: ECommandListType,
    ) -> Result<SCommandQueue, &'static str> {
        // -- $$$FRK(FUTURE WORK): pass priority, flags, nodemask
        let desc = win::D3D12_COMMAND_QUEUE_DESC {
            Type: type_.d3dtype(),
            Priority: win::D3D12_COMMAND_QUEUE_PRIORITY_NORMAL.0,
            Flags: win::D3D12_COMMAND_QUEUE_FLAGS::default(),
            NodeMask: 0,
        };

        let hr = unsafe {
            self.device.CreateCommandQueue::<win::ID3D12CommandQueue>(
                &desc,
            )
        };

        returnerrifwinerror!(hr, "Could not create command queue");

        Ok(SCommandQueue::new_from_raw(hr.expect("checked err above")))
    }

    pub fn create_descriptor_heap(
        &self,
        desc: &SDescriptorHeapDesc,
    ) -> Result<SDescriptorHeap, &'static str> {
        let d3ddesc = desc.d3dtype();

        let hr = unsafe {
            self.device.CreateDescriptorHeap::<win::ID3D12DescriptorHeap>(
                &d3ddesc,
            )
        };

        returnerrifwinerror!(hr, "Failed to create descriptor heap");

        unsafe {
            let heap = hr.expect("checked err above");
            Ok(SDescriptorHeap::new_from_raw(desc.type_, heap))
        }
    }

    pub fn getdescriptorhandleincrementsize(&self, type_: EDescriptorHeapType) -> usize {
        unsafe {
            self.device
                .GetDescriptorHandleIncrementSize(type_.d3dtype()) as usize
        }
    }

    // -- $$$FRK(FUTURE WORK): allow pDesc parameter
    pub fn createrendertargetview(
        &self,
        resource: &SResource,
        destdescriptor: &SCPUDescriptorHandle,
    ) {
        unsafe {
            self.device.CreateRenderTargetView(
                resource.raw(),
                ptr::null(),
                *destdescriptor.raw(),
            );
        }
    }

    pub fn create_depth_stencil_view(
        &self,
        resource: &SResource,
        desc: &SDepthStencilViewDesc,
        dest_descriptor: SCPUDescriptorHandle,
    ) {
        unsafe {
            let d3ddesc = desc.d3dtype();

            self.device.CreateDepthStencilView(
                resource.raw(),
                &d3ddesc,
                *dest_descriptor.raw(),
            );
        }
    }

    pub fn create_shader_resource_view(
        &self,
        resource: &SResource,
        desc: &SShaderResourceViewDesc,
        dest_descriptor: SCPUDescriptorHandle,
    ) {
        unsafe {
            let d3ddesc = desc.d3dtype();

            self.device.CreateShaderResourceView(
                resource.raw(),
                &d3ddesc,
                *dest_descriptor.raw(),
            );
        }
    }

    pub fn create_unordered_access_view(
        &self,
        resource: &SResource,
        desc: &SUnorderedAccessViewDesc,
        dest_descriptor: SCPUDescriptorHandle,
    ) {
        unsafe {
            let d3ddesc = desc.d3dtype();

            self.device.CreateUnorderedAccessView(
                resource.raw(),
                None,
                &d3ddesc,
                *dest_descriptor.raw(),
            );
        }
    }

    pub fn create_committed_resource(
        &self,
        heapproperties: SHeapProperties,
        heapflags: EHeapFlags,
        resourcedesc: SResourceDesc,
        initialresourcestate: EResourceStates,
        clear_value: Option<SClearValue>,
    ) -> Result<SResource, &'static str> {
        unsafe {
            #[allow(unused_assignments)]
            let d3dcv = clear_value.map(|cv| cv.d3dtype());
            let d3dcv_ptr = d3dcv.as_ref().map_or(ptr::null(), |cv| cv);

            let hn = self.device.CreateCommittedResource::<win::ID3D12Resource>(
                heapproperties.raw(),
                heapflags.rawtype(),
                resourcedesc.raw(),
                initialresourcestate.d3dtype(),
                d3dcv_ptr,
            );

            returnerrifwinerror!(hn, "Could not create committed resource.");
            Ok(SResource::new_from_raw(hn.expect("checked err above")))
        }
    }

    pub fn createcommandallocator(
        &self,
        type_: ECommandListType,
    ) -> Result<SCommandAllocator, &'static str> {
        let hn = unsafe {
            self.device.CreateCommandAllocator::<win::ID3D12CommandAllocator>(
                type_.d3dtype(),
            )
        };

        returnerrifwinerror!(hn, "Could not create command allocator.");

        Ok(unsafe { SCommandAllocator::new_from_raw(type_, hn.expect("checked err above")) })
    }

    pub fn createcommandlist(
        &self,
        allocator: &SCommandAllocator,
    ) -> Result<SCommandList, &'static str> {
        let hn = unsafe {
            self.device.CreateCommandList(
                0,
                allocator.type_().d3dtype(),
                allocator.raw(),
                None,
            )
        };

        returnerrifwinerror!(hn, "Could not create command list.");

        Ok(unsafe { SCommandList::new_from_raw(hn.expect("checked err above")) })
    }

    pub fn createfence(&self) -> Result<SFence, &'static str> {
        let hn = unsafe {
            // -- $$$FRK(TODO): support parameters
            self.device.CreateFence::<win::ID3D12Fence>(
                0,
                win::D3D12_FENCE_FLAG_NONE,
            )
        };

        returnerrifwinerror!(hn, "Could not create fence.");

        Ok(unsafe { SFence::new_from_raw(hn.expect("checked err above")) })
    }

    // -- $$$FRK(FUTURE WORK): support nodeMask parameter
    pub fn create_root_signature(
        &self,
        blob_with_root_signature: &SBlob,
    ) -> Result<SRootSignature, &'static str> {
        let hr = unsafe {
            self.device.CreateRootSignature::<win::ID3D12RootSignature>(
                0,
                blob_with_root_signature.raw.GetBufferPointer(),
                blob_with_root_signature.raw.GetBufferSize(),
            )
        };
        returnerrifwinerror!(hr, "Could not create root signature");

        let root_signature = hr.expect("checked err above");
        Ok(SRootSignature {
            raw: root_signature,
        })
    }

    pub fn create_pipeline_state_for_raw_desc(
        &self,
        desc: &win::D3D12_PIPELINE_STATE_STREAM_DESC,
    ) -> Result<SPipelineState, &'static str> {
        let hr = unsafe {
            self.device.CreatePipelineState::<win::ID3D12PipelineState>(
                desc,
            )
        };
        returnerrifwinerror!(hr, "Could not create pipeline state");

        unsafe {
            let pipeline_state = hr.expect("checked err above");
            Ok(SPipelineState::new_from_raw(pipeline_state))
        }
    }

    pub fn create_pipeline_state<T>(
        &self,
        desc: &SPipelineStateStreamDesc<T>,
    ) -> Result<SPipelineState, &'static str> {
        let d3ddesc = unsafe { desc.d3dtype() };
        self.create_pipeline_state_for_raw_desc(&d3ddesc)
    }

    pub fn copy_descriptors(
        &self,
        dest_descriptor_range_starts: &[SCPUDescriptorHandle],
        dest_descriptor_range_sizes: &[u32],
        src_descriptor_range_starts: &[SCPUDescriptorHandle],
        src_descriptor_range_sizes: &[u32],
        type_: EDescriptorHeapType,
    ) {
        use std::mem::size_of;

        assert_eq!(
            dest_descriptor_range_starts.len(),
            dest_descriptor_range_sizes.len()
        );
        assert_eq!(
            src_descriptor_range_starts.len(),
            src_descriptor_range_sizes.len()
        );
        assert_eq!(
            size_of::<win::D3D12_CPU_DESCRIPTOR_HANDLE>(),
            size_of::<SCPUDescriptorHandle>()
        );

        // -- Note: SCPUDescriptorHandle is repr(C) and just holds a D3D12_CPU_HANDLE...

        let dest_starts_ptr: *const SCPUDescriptorHandle = dest_descriptor_range_starts.as_ptr();

        unsafe {
            self.device.CopyDescriptors(
                dest_descriptor_range_sizes.len() as u32,
                dest_starts_ptr as *const win::D3D12_CPU_DESCRIPTOR_HANDLE,
                dest_descriptor_range_sizes.as_ptr(),
                src_descriptor_range_sizes.len() as u32,
                src_descriptor_range_starts.as_ptr() as *const win::D3D12_CPU_DESCRIPTOR_HANDLE,
                src_descriptor_range_sizes.as_ptr(),
                type_.d3dtype(),
            );
        }
    }
}

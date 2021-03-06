use super::*;

#[derive(Clone)]
pub struct SDevice {
    device: ComPtr<ID3D12Device2>,
}

pub fn d3d12createdevice(adapter: *mut unknwnbase::IUnknown) -> Result<SDevice, &'static str> {
    let mut rawdevice: *mut ID3D12Device2 = ptr::null_mut();
    let hn = unsafe {
        D3D12CreateDevice(
            adapter, //self.adapter.asunknownptr(),
            d3dcommon::D3D_FEATURE_LEVEL_11_0,
            &ID3D12Device2::uuidof(),
            &mut rawdevice as *mut *mut _ as *mut *mut c_void,
        )
    };
    returnerrifwinerror!(hn, "Could not create device on adapter.");

    let device = unsafe { ComPtr::from_raw(rawdevice) };
    Ok(SDevice { device: device })
}

impl SDevice {
    pub fn castinfoqueue(&self) -> Option<SInfoQueue> {
        match self.device.cast::<ID3D12InfoQueue>() {
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
        let desc = D3D12_COMMAND_QUEUE_DESC {
            Type: type_.d3dtype(),
            Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL as i32,
            Flags: 0,
            NodeMask: 0,
        };

        let mut rawqueue: *mut ID3D12CommandQueue = ptr::null_mut();
        let hr = unsafe {
            self.device.CreateCommandQueue(
                &desc,
                &ID3D12CommandQueue::uuidof(),
                &mut rawqueue as *mut *mut _ as *mut *mut c_void,
            )
        };

        returnerrifwinerror!(hr, "Could not create command queue");

        Ok(unsafe { SCommandQueue::new_from_raw(ComPtr::from_raw(rawqueue)) })
    }

    pub fn create_descriptor_heap(
        &self,
        desc: &SDescriptorHeapDesc,
    ) -> Result<SDescriptorHeap, &'static str> {
        let d3ddesc = desc.d3dtype();

        let mut rawheap: *mut ID3D12DescriptorHeap = ptr::null_mut();
        let hr = unsafe {
            self.device.CreateDescriptorHeap(
                &d3ddesc,
                &ID3D12DescriptorHeap::uuidof(),
                &mut rawheap as *mut *mut _ as *mut *mut c_void,
            )
        };

        returnerrifwinerror!(hr, "Failed to create descriptor heap");

        unsafe {
            let heap = ComPtr::from_raw(rawheap);
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
                resource.raw().as_raw(),
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
                resource.raw().as_raw(),
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
                resource.raw().as_raw(),
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
                resource.raw().as_raw(),
                ptr::null_mut(),
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

            let mut rawresource: *mut ID3D12Resource = ptr::null_mut();
            let hn = self.device.CreateCommittedResource(
                heapproperties.raw(),
                heapflags.rawtype(),
                resourcedesc.raw(),
                initialresourcestate.d3dtype(),
                d3dcv_ptr,
                &ID3D12Resource::uuidof(), // $$$FRK(TODO): this isn't necessarily right
                &mut rawresource as *mut *mut _ as *mut *mut c_void,
            );

            returnerrifwinerror!(hn, "Could not create committed resource.");
            Ok(SResource::new_from_raw(ComPtr::from_raw(rawresource)))
        }
    }

    pub fn createcommandallocator(
        &self,
        type_: ECommandListType,
    ) -> Result<SCommandAllocator, &'static str> {
        let mut rawca: *mut ID3D12CommandAllocator = ptr::null_mut();
        let hn = unsafe {
            self.device.CreateCommandAllocator(
                type_.d3dtype(),
                &ID3D12CommandAllocator::uuidof(),
                &mut rawca as *mut *mut _ as *mut *mut c_void,
            )
        };

        returnerrifwinerror!(hn, "Could not create command allocator.");

        Ok(unsafe { SCommandAllocator::new_from_raw(type_, ComPtr::from_raw(rawca)) })
    }

    pub fn createcommandlist(
        &self,
        allocator: &SCommandAllocator,
    ) -> Result<SCommandList, &'static str> {
        let mut rawcl: *mut ID3D12GraphicsCommandList = ptr::null_mut();
        let hn = unsafe {
            self.device.CreateCommandList(
                0,
                allocator.type_().d3dtype(),
                allocator.raw().as_raw(),
                ptr::null_mut(),
                &ID3D12GraphicsCommandList::uuidof(),
                &mut rawcl as *mut *mut _ as *mut *mut c_void,
            )
        };

        returnerrifwinerror!(hn, "Could not create command list.");

        Ok(unsafe { SCommandList::new_from_raw(ComPtr::from_raw(rawcl)) })
    }

    pub fn createfence(&self) -> Result<SFence, &'static str> {
        let mut rawf: *mut ID3D12Fence = ptr::null_mut();
        let hn = unsafe {
            // -- $$$FRK(TODO): support parameters
            self.device.CreateFence(
                0,
                D3D12_FENCE_FLAG_NONE,
                &ID3D12Fence::uuidof(),
                &mut rawf as *mut *mut _ as *mut *mut c_void,
            )
        };

        returnerrifwinerror!(hn, "Could not create fence.");

        Ok(unsafe { SFence::new_from_raw(ComPtr::from_raw(rawf)) })
    }

    // -- $$$FRK(FUTURE WORK): support nodeMask parameter
    pub fn create_root_signature(
        &self,
        blob_with_root_signature: &SBlob,
    ) -> Result<SRootSignature, &'static str> {
        let mut raw_root_signature: *mut ID3D12RootSignature = ptr::null_mut();

        let hr = unsafe {
            self.device.CreateRootSignature(
                0,
                blob_with_root_signature.raw.GetBufferPointer(),
                blob_with_root_signature.raw.GetBufferSize(),
                &ID3D12RootSignature::uuidof(),
                &mut raw_root_signature as *mut *mut _ as *mut *mut c_void,
            )
        };
        returnerrifwinerror!(hr, "Could not create root signature");

        let root_signature = unsafe { ComPtr::from_raw(raw_root_signature) };
        Ok(SRootSignature {
            raw: root_signature,
        })
    }

    pub fn create_pipeline_state_for_raw_desc(
        &self,
        desc: &D3D12_PIPELINE_STATE_STREAM_DESC,
    ) -> Result<SPipelineState, &'static str> {
        let mut raw_pipeline_state: *mut ID3D12PipelineState = ptr::null_mut();

        let hr = unsafe {
            self.device.CreatePipelineState(
                desc,
                &ID3D12PipelineState::uuidof(),
                &mut raw_pipeline_state as *mut *mut _ as *mut *mut c_void,
            )
        };
        returnerrifwinerror!(hr, "Could not create pipeline state");

        unsafe {
            let pipeline_state = ComPtr::from_raw(raw_pipeline_state);
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
            size_of::<D3D12_CPU_DESCRIPTOR_HANDLE>(),
            size_of::<SCPUDescriptorHandle>()
        );
        assert_eq!(size_of::<UINT>(), size_of::<u32>());

        // -- Note: SCPUDescriptorHandle is repr(C) and just holds a D3D12_CPU_HANDLE...

        let dest_starts_ptr: *const SCPUDescriptorHandle = dest_descriptor_range_starts.as_ptr();

        unsafe {
            self.device.CopyDescriptors(
                dest_descriptor_range_sizes.len() as UINT,
                dest_starts_ptr as *const D3D12_CPU_DESCRIPTOR_HANDLE,
                dest_descriptor_range_sizes.as_ptr(),
                src_descriptor_range_sizes.len() as UINT,
                src_descriptor_range_starts.as_ptr() as *const D3D12_CPU_DESCRIPTOR_HANDLE,
                src_descriptor_range_sizes.as_ptr(),
                type_.d3dtype(),
            );
        }
    }
}

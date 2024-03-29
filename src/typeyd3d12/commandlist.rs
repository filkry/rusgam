use arrayvec::{ArrayVec};

use super::*;

#[derive(Copy, Clone, PartialEq)]
pub enum ECommandListType {
    Invalid,
    Direct,
    Bundle,
    Compute,
    Copy,
    //VideoDecode,
    //VideoProcess,
}

impl ECommandListType {
    pub fn d3dtype(&self) -> win::D3D12_COMMAND_LIST_TYPE {
        match self {
            ECommandListType::Invalid => panic!("Trying to use invalid CommandListType"),
            ECommandListType::Direct => win::D3D12_COMMAND_LIST_TYPE_DIRECT,
            ECommandListType::Bundle => win::D3D12_COMMAND_LIST_TYPE_BUNDLE,
            ECommandListType::Compute => win::D3D12_COMMAND_LIST_TYPE_COMPUTE,
            ECommandListType::Copy => win::D3D12_COMMAND_LIST_TYPE_COPY,
            //VideoDecode => D3D12_COMMAND_LIST_TYPE_VIDEO_DECODE ,
            //VideoProcess => D3D12_COMMAND_LIST_TYPE_VIDEO_PROCESS ,
        }
    }

    pub fn new_from_d3dtype(d3dtype: win::D3D12_COMMAND_LIST_TYPE) -> Self {
        match d3dtype {
            win::D3D12_COMMAND_LIST_TYPE_DIRECT => ECommandListType::Direct,
            win::D3D12_COMMAND_LIST_TYPE_BUNDLE => ECommandListType::Bundle,
            win::D3D12_COMMAND_LIST_TYPE_COMPUTE => ECommandListType::Compute,
            win::D3D12_COMMAND_LIST_TYPE_COPY => ECommandListType::Copy,
            _ => ECommandListType::Invalid,
        }
    }
}

#[derive(Clone)]
pub struct SCommandList {
    commandlist: win::ID3D12GraphicsCommandList,
}

impl SCommandList {
    pub unsafe fn new_from_raw(raw: win::ID3D12GraphicsCommandList) -> Self {
        Self { commandlist: raw }
    }

    // -- almost everything in here is unsafe because we take shared references, but require
    // -- exclusive access to be thread safe

    pub fn gettype(&self) -> ECommandListType {
        unsafe { ECommandListType::new_from_d3dtype(self.commandlist.GetType()) }
    }

    pub unsafe fn reset(&self, commandallocator: &SCommandAllocator) -> Result<(), &'static str> {
        let hn = self
            .commandlist
            .Reset(commandallocator.raw(), None);
        match hn {
            Ok(_) => Ok(()),
            Err(_) => Err("Could not reset command list."),
        }
    }

    pub unsafe fn resource_barrier<const N: usize>(&self, barriers: [SBarrier; N]) {
        let mut raw_barriers = ArrayVec::<[win::D3D12_RESOURCE_BARRIER; 10]>::new();
        for barrier in std::array::IntoIter::new(barriers) {
            raw_barriers.push(barrier.barrier);
        }

        self.commandlist.ResourceBarrier(1, raw_barriers.as_ptr());
    }

    pub fn copy_buffer_region(
        &mut self,
        dst_buffer: &SResource,
        dst_offset: usize,
        src_buffer: &SResource,
        src_offset: usize,
        num_bytes: usize,
    ) {
        self.commandlist.CopyBufferRegion(
            dst_buffer.raw(),
            dst_offset as u64,
            src_buffer.raw(),
            src_offset as u64,
            num_bytes as u64,
        );
    }

    pub unsafe fn clearrendertargetview(
        &self,
        descriptor: SCPUDescriptorHandle,
        colour: &[f32; 4],
    ) {
        // -- $$$FRK(FUTURE WORK): support third/fourth parameter
        self.commandlist
            .ClearRenderTargetView(*descriptor.raw(), &colour[0], 0, ptr::null());
    }

    pub unsafe fn clear_depth_stencil_view(&self, descriptor: SCPUDescriptorHandle, depth: f32) {
        // -- $$$FRK(FUTURE WORK): support ClearFlags/Stencil/NumRects/pRects
        self.commandlist.ClearDepthStencilView(
            *descriptor.raw(),
            win::D3D12_CLEAR_FLAG_DEPTH,
            depth,
            0,
            0,
            ptr::null(),
        );
    }

    pub unsafe fn set_pipeline_state(&self, pipeline_state: &SPipelineState) {
        self.commandlist
            .SetPipelineState(pipeline_state.raw())
    }

    pub unsafe fn set_graphics_root_signature(&self, root_signature: &SRootSignature) {
        self.commandlist
            .SetGraphicsRootSignature(root_signature.raw.clone())
    }

    pub unsafe fn set_compute_root_signature(&self, root_signature: &SRootSignature) {
        self.commandlist
            .SetComputeRootSignature(root_signature.raw.clone())
    }

    pub unsafe fn ia_set_primitive_topology(&self, primitive_topology: EPrimitiveTopology) {
        self.commandlist
            .IASetPrimitiveTopology(primitive_topology.d3dtype())
    }

    pub unsafe fn ia_set_vertex_buffers(
        &self,
        start_slot: u32,
        vertex_buffers: &[&SVertexBufferView],
    ) {
        assert!(vertex_buffers.len() <= 10);

        let mut raw_array : [win::D3D12_VERTEX_BUFFER_VIEW ; 10] = [
            std::mem::MaybeUninit::zeroed().assume_init(),
            std::mem::MaybeUninit::zeroed().assume_init(),
            std::mem::MaybeUninit::zeroed().assume_init(),
            std::mem::MaybeUninit::zeroed().assume_init(),
            std::mem::MaybeUninit::zeroed().assume_init(),
            std::mem::MaybeUninit::zeroed().assume_init(),
            std::mem::MaybeUninit::zeroed().assume_init(),
            std::mem::MaybeUninit::zeroed().assume_init(),
            std::mem::MaybeUninit::zeroed().assume_init(),
            std::mem::MaybeUninit::zeroed().assume_init(),
        ];

        for (i, vb) in vertex_buffers.iter().enumerate() {
            raw_array[i] = *vb.raw();
        }

        self.commandlist.IASetVertexBuffers(
            start_slot,
            vertex_buffers.len() as u32,
            &raw_array[0],
        )
    }

    pub unsafe fn ia_set_index_buffer(&self, index_buffer: &SIndexBufferView) {
        self.commandlist.IASetIndexBuffer(index_buffer.raw())
    }

    pub unsafe fn rs_set_viewports(&self, viewports: &[&SViewport]) {
        assert!(viewports.len() == 1); // didn't want to implement copying d3dtype array
        self.commandlist
            .RSSetViewports(viewports.len() as u32, &viewports[0].viewport)
    }

    pub unsafe fn rs_set_scissor_rects(&self, scissor_rects: SScissorRects) {
        self.commandlist.RSSetScissorRects(
            scissor_rects.d3drects.len() as u32,
            &scissor_rects.d3drects[0],
        )
    }

    pub unsafe fn om_set_render_targets(
        &self,
        render_target_descriptors: &[&SCPUDescriptorHandle],
        rts_single_handle_to_descriptor_range: bool,
        depth_target_descriptor: &SCPUDescriptorHandle,
    ) {
        assert!(render_target_descriptors.len() <= 1); // didn't want to implement copying d3dtype array

        if render_target_descriptors.len() == 1 {
            self.commandlist.OMSetRenderTargets(
                render_target_descriptors.len() as u32,
                render_target_descriptors[0].raw(),
                rts_single_handle_to_descriptor_range,
                depth_target_descriptor.raw(),
            );
        }
        else if render_target_descriptors.len() == 0 {
            self.commandlist.OMSetRenderTargets(
                0, ptr::null_mut(), rts_single_handle_to_descriptor_range,
                depth_target_descriptor.raw(),
            );
        }
    }

    pub unsafe fn set_descriptor_heaps(&self, heaps: &[&SDescriptorHeap]) {
        if heaps.len() > 0 {
            let mut raw_heaps = ArrayVec::<[Option<win::ID3D12DescriptorHeap>; 4]>::new();
            for heap in heaps.iter() {
                raw_heaps.push(Some(heap.heap.clone()));
            }

            self.commandlist
                .SetDescriptorHeaps(heaps.len() as u32, &mut raw_heaps[0]);
        }
    }

    pub unsafe fn set_graphics_root_32_bit_constants<T: Sized>(
        &self,
        root_parameter_index: u32,
        data: &T,
        dest_offset_in_32_bit_values: u32,
    ) {
        let num_values = mem::size_of::<T>() / 4;
        let src_data_ptr = data as *const T as *const std::ffi::c_void;

        self.commandlist.SetGraphicsRoot32BitConstants(
            root_parameter_index,
            num_values as u32,
            src_data_ptr,
            dest_offset_in_32_bit_values,
        );
    }

    pub unsafe fn set_graphics_root_shader_resource_view(
        &self,
        root_parameter_index: u32,
        buffer_location: SGPUDescriptorHandle,
    ) {
        self.commandlist.SetGraphicsRootShaderResourceView(
            root_parameter_index,
            buffer_location.raw().ptr,
        );
    }

    pub unsafe fn set_graphics_root_descriptor_table(
        &self,
        root_parameter_index: usize,
        base_descriptor: &SGPUDescriptorHandle,
    ) {
        self.commandlist.SetGraphicsRootDescriptorTable(
            root_parameter_index as u32,
            base_descriptor.d3dtype(),
        );
    }

    pub unsafe fn set_compute_root_shader_resource_view(
        &self,
        root_parameter_index: u32,
        buffer_location: SGPUVirtualAddress,
    ) {
        self.commandlist.SetComputeRootShaderResourceView(
            root_parameter_index,
            buffer_location.raw(),
        );
    }

    pub unsafe fn set_compute_root_unordered_access_view(
        &self,
        root_parameter_index: u32,
        buffer_location: SGPUVirtualAddress,
    ) {
        self.commandlist.SetComputeRootUnorderedAccessView(
            root_parameter_index,
            buffer_location.raw(),
        );
    }

    pub unsafe fn draw_indexed_instanced(
        &self,
        index_count_per_instance: u32,
        instance_count: u32,
        start_index_location: u32,
        base_vertex_location: i32,
        start_instance_location: u32,
    ) {
        self.commandlist.DrawIndexedInstanced(
            index_count_per_instance,
            instance_count,
            start_index_location,
            base_vertex_location,
            start_instance_location,
        );
    }

    pub unsafe fn draw_instanced(
        &self,
        vertex_count_per_instance: u32,
        instance_count: u32,
        base_vertex_location: u32,
        start_instance_location: u32,
    ) {
        self.commandlist.DrawInstanced(
            vertex_count_per_instance,
            instance_count,
            base_vertex_location,
            start_instance_location,
        );
    }

    pub unsafe fn dispatch(
        &self,
        thread_group_count_x : u32,
        thread_group_count_y : u32,
        thread_group_count_z : u32,
    ) {
        self.commandlist.Dispatch(
            thread_group_count_x,
            thread_group_count_y,
            thread_group_count_z,
        );
    }

    pub unsafe fn close(&self) -> Result<(), &'static str> {
        let hn = self.commandlist.Close();
        match hn {
            Ok(_) => Ok(()),
            Err(_) => Err("Could not close command list."),
        }
    }

    pub unsafe fn raw(&self) -> &win::ID3D12GraphicsCommandList {
        &self.commandlist
    }

    pub unsafe fn raw_mut(&mut self) -> &mut win::ID3D12GraphicsCommandList {
        &mut self.commandlist
    }
}

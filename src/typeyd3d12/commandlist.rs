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
    pub fn d3dtype(&self) -> D3D12_COMMAND_LIST_TYPE {
        match self {
            ECommandListType::Invalid => D3D12_COMMAND_LIST_TYPE_DIRECT, // $$$FRK(TODO): obviously wrong, this needs to return an option I guess
            ECommandListType::Direct => D3D12_COMMAND_LIST_TYPE_DIRECT,
            ECommandListType::Bundle => D3D12_COMMAND_LIST_TYPE_BUNDLE,
            ECommandListType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
            ECommandListType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
            //VideoDecode => D3D12_COMMAND_LIST_TYPE_VIDEO_DECODE ,
            //VideoProcess => D3D12_COMMAND_LIST_TYPE_VIDEO_PROCESS ,
        }
    }

    pub fn new_from_d3dtype(d3dtype: D3D12_COMMAND_LIST_TYPE) -> Self {
        match d3dtype {
            D3D12_COMMAND_LIST_TYPE_DIRECT => ECommandListType::Direct,
            D3D12_COMMAND_LIST_TYPE_BUNDLE => ECommandListType::Bundle,
            D3D12_COMMAND_LIST_TYPE_COMPUTE => ECommandListType::Compute,
            D3D12_COMMAND_LIST_TYPE_COPY => ECommandListType::Copy,
            _ => ECommandListType::Invalid,
        }
    }
}

#[derive(Clone)]
pub struct SCommandList {
    commandlist: ComPtr<ID3D12GraphicsCommandList>,
}

impl SCommandList {
    pub unsafe fn new_from_raw(raw: ComPtr<ID3D12GraphicsCommandList>) -> Self {
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
            .Reset(commandallocator.raw().as_raw(), ptr::null_mut());
        returnerrifwinerror!(hn, "Could not reset command list.");
        Ok(())
    }

    pub unsafe fn resourcebarrier(&self, numbarriers: u32, barriers: &[SBarrier]) {
        // -- $$$FRK(TODO): need to figure out how to make a c array from the rust slice
        // -- w/o a heap allocation...
        assert!(numbarriers == 1);
        self.commandlist.ResourceBarrier(1, &(barriers[0].barrier));
    }

    pub unsafe fn clearrendertargetview(
        &self,
        descriptor: SCPUDescriptorHandle,
        colour: &[f32; 4],
    ) {
        // -- $$$FRK(TODO): support third/fourth parameter
        self.commandlist
            .ClearRenderTargetView(*descriptor.raw(), colour, 0, ptr::null());
    }

    pub unsafe fn clear_depth_stencil_view(&self, descriptor: SCPUDescriptorHandle, depth: f32) {
        // -- $$$FRK(TODO): support ClearFlags/Stencil/NumRects/pRects
        self.commandlist.ClearDepthStencilView(
            *descriptor.raw(),
            D3D12_CLEAR_FLAG_DEPTH,
            depth,
            0,
            0,
            ptr::null(),
        );
    }

    pub unsafe fn set_pipeline_state(&self, pipeline_state: &SPipelineState) {
        self.commandlist
            .SetPipelineState(pipeline_state.raw().as_raw())
    }

    pub unsafe fn set_graphics_root_signature(&self, root_signature: &SRootSignature) {
        self.commandlist
            .SetGraphicsRootSignature(root_signature.raw.as_raw())
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
        assert!(vertex_buffers.len() == 1); // didn't want to implement copying d3dtype array
        self.commandlist.IASetVertexBuffers(
            start_slot,
            vertex_buffers.len() as u32,
            vertex_buffers[0].raw(),
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
                rts_single_handle_to_descriptor_range as i32,
                depth_target_descriptor.raw(),
            );
        }
        else if render_target_descriptors.len() == 0 {
            self.commandlist.OMSetRenderTargets(
                0, ptr::null_mut(), rts_single_handle_to_descriptor_range as i32,
                depth_target_descriptor.raw(),
            );
        }
    }

    pub unsafe fn set_descriptor_heaps(&self, heaps: &[&SDescriptorHeap]) {
        if heaps.len() > 0 {
            let mut raw_heaps = [ptr::null_mut(); 4]; // only 4 heap types
            for (i, heap) in heaps.iter().enumerate() {
                raw_heaps[i] = heap.heap.as_raw();
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
        let src_data_ptr = data as *const T as *const c_void;

        self.commandlist.SetGraphicsRoot32BitConstants(
            root_parameter_index,
            num_values as UINT,
            src_data_ptr,
            dest_offset_in_32_bit_values,
        );
    }

    /*
    pub unsafe fn set_graphics_root_shader_resource_view(
        &self,
        root_parameter_index: u32,
        buffer_location: SGPUDescriptorHandle,
    ) {
        self.commandlist.SetGraphicsRootShaderResourceView(
            root_parameter_index,
            buffer_location.raw(),
        );
    }
    */

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

    pub unsafe fn close(&self) -> Result<(), &'static str> {
        let hn = self.commandlist.Close();
        returnerrifwinerror!(hn, "Could not close command list.");
        Ok(())
    }

    pub unsafe fn set_graphics_root_descriptor_table(
        &self,
        root_parameter_index: usize,
        base_descriptor: &SGPUDescriptorHandle,
    ) {
        self.commandlist.SetGraphicsRootDescriptorTable(
            root_parameter_index as UINT,
            base_descriptor.d3dtype(),
        );
    }

    pub unsafe fn raw(&self) -> &ComPtr<ID3D12GraphicsCommandList> {
        &self.commandlist
    }

    pub unsafe fn rawmut(&mut self) -> &mut ComPtr<ID3D12GraphicsCommandList> {
        &mut self.commandlist
    }
}

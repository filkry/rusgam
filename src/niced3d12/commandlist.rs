use super::*;

pub struct SCommandList {
    raw: t12::SCommandList,
    //allocator: &RefCell<t12::SCommandAllocator>,
}

impl SCommandList {
    pub fn new_from_raw(raw: t12::SCommandList) -> Self {
        Self { raw: raw }
    }

    pub fn raw(&self) -> &t12::SCommandList {
        &self.raw
    }

    // -- by default, unsafe blocks are here because we are guaranteeing exclusive access to
    // -- the CommandList via the &mut self reference

    pub fn reset(&mut self, allocator: &mut SCommandAllocator) -> Result<(), &'static str> {
        unsafe { self.raw.reset(&allocator.raw()) }
    }

    pub fn transition_resource(
        &mut self,
        resource: &SResource,
        beforestate: t12::EResourceStates,
        afterstate: t12::EResourceStates,
    ) -> Result<(), &'static str> {
        let transbarrier = t12::create_transition_barrier(&resource.raw, beforestate, afterstate);
        unsafe { self.raw.resourcebarrier(1, &[transbarrier]) };
        Ok(())
    }

    pub fn clear_render_target_view(
        &mut self,
        rtvdescriptor: t12::SCPUDescriptorHandle,
        colour: &[f32; 4],
    ) -> Result<(), &'static str> {
        unsafe { self.raw.clearrendertargetview(rtvdescriptor, colour) };
        Ok(())
    }

    pub fn clear_depth_stencil_view(
        &mut self,
        dsv_descriptor: t12::SCPUDescriptorHandle,
        depth: f32,
    ) -> Result<(), &'static str> {
        unsafe { self.raw.clear_depth_stencil_view(dsv_descriptor, depth) };
        Ok(())
    }

    pub fn set_pipeline_state(&mut self, pipeline_state: &t12::SPipelineState) {
        unsafe { self.raw.set_pipeline_state(pipeline_state) }
    }

    pub fn set_graphics_root_signature(&mut self, root_signature: &t12::SRootSignature) {
        unsafe { self.raw.set_graphics_root_signature(root_signature) }
    }

    pub fn ia_set_primitive_topology(&mut self, primitive_topology: t12::EPrimitiveTopology) {
        unsafe { self.raw.ia_set_primitive_topology(primitive_topology) }
    }

    pub fn ia_set_vertex_buffers(
        &mut self,
        start_slot: u32,
        vertex_buffers: &[&t12::SVertexBufferView],
    ) {
        unsafe { self.raw.ia_set_vertex_buffers(start_slot, vertex_buffers) }
    }

    pub fn ia_set_index_buffer(&mut self, index_buffer: &t12::SIndexBufferView) {
        unsafe { self.raw.ia_set_index_buffer(index_buffer) }
    }

    pub fn rs_set_viewports(&mut self, viewports: &[&t12::SViewport]) {
        unsafe { self.raw.rs_set_viewports(viewports) }
    }

    pub fn rs_set_scissor_rects(&mut self, scissor_rects: t12::SScissorRects) {
        unsafe { self.raw.rs_set_scissor_rects(scissor_rects) }
    }

    pub fn om_set_render_targets(
        &self,
        render_target_descriptors: &[&t12::SCPUDescriptorHandle],
        rts_single_handle_to_descriptor_range: bool,
        depth_target_descriptor: &t12::SCPUDescriptorHandle,
    ) {
        unsafe {
            self.raw.om_set_render_targets(
                render_target_descriptors,
                rts_single_handle_to_descriptor_range,
                depth_target_descriptor,
            )
        };
    }

    pub fn set_graphics_root_32_bit_constants<T: Sized>(
        &mut self,
        root_parameter_index: u32,
        data: &T,
        dest_offset_in_32_bit_values: u32,
    ) {
        unsafe {
            self.raw.set_graphics_root_32_bit_constants(
                root_parameter_index,
                data,
                dest_offset_in_32_bit_values,
            )
        };
    }

    pub fn draw_indexed_instanced(
        &mut self,
        index_count_per_instance: u32,
        instance_count: u32,
        start_index_location: u32,
        base_vertex_location: i32,
        start_instance_location: u32,
    ) {
        unsafe {
            self.raw.draw_indexed_instanced(
                index_count_per_instance,
                instance_count,
                start_index_location,
                base_vertex_location,
                start_instance_location,
            )
        };
    }

    pub fn get_type(&self) -> t12::ECommandListType {
        self.raw.gettype()
    }

    pub fn close(&mut self) -> Result<(), &'static str> {
        unsafe { self.raw.close() }
    }

    pub fn set_graphics_root_descriptor_table(
        &mut self,
        root_parameter_index: usize,
        base_descriptor: &t12::SGPUDescriptorHandle
    ) {
        unsafe { self.raw.set_graphics_root_descriptor_table(root_parameter_index, base_descriptor) };
    }

    pub fn update_buffer_resource<T>(
        &mut self,
        device: &SDevice,
        bufferdata: &[T],
        flags: t12::SResourceFlags,
    ) -> Result<SCommandQueueUpdateBufferResult, &'static str> {
        let mut destinationresource = device.create_committed_buffer_resource_for_data(
            t12::EHeapType::Default,
            flags,
            t12::EResourceStates::CopyDest,
            bufferdata,
        )?;

        // -- resource created with Upload type MUST have state GenericRead
        let mut intermediateresource = device.create_committed_buffer_resource_for_data(
            t12::EHeapType::Upload,
            flags,
            t12::EResourceStates::GenericRead,
            bufferdata,
        )?;

        let mut srcdata = t12::SSubResourceData::createbuffer(bufferdata);
        update_subresources_stack(
            self,
            &mut destinationresource,
            &mut intermediateresource,
            0,
            0,
            1,
            &mut srcdata,
        );

        Ok(SCommandQueueUpdateBufferResult {
            destinationresource: destinationresource,
            intermediateresource: intermediateresource,
        })
    }
}

pub struct SCommandQueueUpdateBufferResult {
    pub destinationresource: SResource,
    pub intermediateresource: SResource,
}

struct SImguiRenderer {
    imgui_font_texture: SPoolHandle,
    imgui_font_texture_id: imgui::TextureId,
    imgui_root_signature: n12::SRootSignature,
    imgui_pipeline_state: t12::SPipelineState,
    imgui_orthomat_root_param_idx: usize,
    imgui_texture_descriptor_table_param_idx: usize,
    _imgui_vert_byte_code: t12::SShaderBytecode,
    _imgui_pixel_byte_code: t12::SShaderBytecode,
    imgui_vert_buffer_resources: [SMemVec::<'a, n12::SResource>; 2],
    imgui_vert_buffer_views: [SMemVec::<'a, t12::SVertexBufferView>; 2],
    imgui_index_buffer_resources: [SMemVec::<'a, n12::SResource>; 2],
    imgui_index_buffer_views: [SMemVec::<'a, t12::SIndexBufferView>; 2],
}
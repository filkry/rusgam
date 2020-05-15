#[allow(dead_code)]
struct SDebugLine {
    start: Vec3,
    end: Vec3,
    colour: Vec3,
    draw_over_world: bool,
}

struct SDynamicRenderer {
    line_pipeline_state: t12::SPipelineState,
    line_root_signature: n12::SRootSignature,
    line_vp_root_param_idx: usize,
    _line_vert_byte_code: t12::SShaderBytecode,
    _line_pixel_byte_code: t12::SShaderBytecode,

    lines: SMemVec::<'a, SDebugLine>,
    line_vertex_buffer_intermediate_resource: [Option<n12::SResource>; 2],
    line_vertex_buffer_resource: [Option<n12::SResource>; 2],
    line_vertex_buffer_view: [Option<t12::SVertexBufferView>; 2],
}
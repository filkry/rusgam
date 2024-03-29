use super::*;

#[derive(Copy, Clone, PartialEq)]
pub enum EInputClassification {
    PerVertexData,
    PerInstanceData,
}

impl EInputClassification {
    pub fn d3dtype(&self) -> win::D3D12_INPUT_CLASSIFICATION {
        match self {
            Self::PerVertexData => win::D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            Self::PerInstanceData => win::D3D12_INPUT_CLASSIFICATION_PER_INSTANCE_DATA,
        }
    }
}

#[derive(Copy, Clone)]
pub struct SInputElementDesc {
    pub semantic_name: &'static str,
    pub semantic_index: u32,
    pub format: EDXGIFormat,
    pub input_slot: u32,
    pub aligned_byte_offset: u32,
    pub input_slot_class: EInputClassification,
    pub instance_data_step_rate: u32,

    semantic_name_null_terminated: [u8; 32],
}

impl SInputElementDesc {
    pub fn create(
        semantic_name: &'static str,
        semantic_index: u32,
        format: EDXGIFormat,
        input_slot: u32,
        aligned_byte_offset: u32,
        input_slot_class: EInputClassification,
        instance_data_step_rate: u32,
    ) -> Self {
        let mut result = Self {
            semantic_name: semantic_name,
            semantic_index: semantic_index,
            format: format,
            input_slot: input_slot,
            aligned_byte_offset: aligned_byte_offset,
            input_slot_class: input_slot_class,
            instance_data_step_rate: instance_data_step_rate,

            semantic_name_null_terminated: [0; 32],
        };

        let mut i = 0;
        for c in semantic_name.as_bytes() {
            result.semantic_name_null_terminated[i] = *c as u8;
            i += 1;
        }
        result.semantic_name_null_terminated[i] = 0;

        result
    }

    pub unsafe fn d3dtype(&mut self) -> win::D3D12_INPUT_ELEMENT_DESC {
        win::D3D12_INPUT_ELEMENT_DESC {
            //SemanticName: self.semantic_name_utf16.as_ptr(),
            SemanticName: win::PSTR(self.semantic_name_null_terminated.as_mut_ptr()),
            SemanticIndex: self.semantic_index,
            Format: self.format.d3dtype(),
            InputSlot: self.input_slot,
            AlignedByteOffset: self.aligned_byte_offset,
            InputSlotClass: self.input_slot_class.d3dtype(),
            InstanceDataStepRate: self.instance_data_step_rate,
        }
    }
}

pub struct SPipelineState {
    raw: win::ID3D12PipelineState,
}

impl SPipelineState {
    pub unsafe fn new_from_raw(raw: win::ID3D12PipelineState) -> Self {
        Self { raw: raw }
    }

    pub unsafe fn raw(&self) -> &win::ID3D12PipelineState {
        &self.raw
    }
}

pub struct SInputLayoutDesc {
    pub input_element_descs: ArrayVec<[SInputElementDesc; 16]>,

    pub d3d_input_element_descs: ArrayVec<[win::D3D12_INPUT_ELEMENT_DESC; 16]>,
}

impl SInputLayoutDesc {
    pub unsafe fn generate_d3dtype(&mut self) {
        self.d3d_input_element_descs.clear();

        for input_element_desc in &mut self.input_element_descs {
            self.d3d_input_element_descs
                .push(input_element_desc.d3dtype());
        }
    }

    pub unsafe fn d3dtype(&mut self) -> win::D3D12_INPUT_LAYOUT_DESC {
        // -- $$$FRK(NOTE): the generate data here is no longer valid if this moves!!!
        // -- it contains internal references!
        self.generate_d3dtype();

        let result = win::D3D12_INPUT_LAYOUT_DESC {
            pInputElementDescs: self.d3d_input_element_descs.as_mut_ptr(),
            NumElements: self.d3d_input_element_descs.len() as u32,
        };

        result
    }
}

pub enum EStencilOp {
    Keep,
    Zero,
    Replace,
    IncrSat,
    DecrSat,
    Invert,
    Incr,
    Decr,
}

impl EStencilOp {
    pub fn d3dtype(&self) -> win::D3D12_STENCIL_OP {
        match self {
            Self::Keep => win::D3D12_STENCIL_OP_KEEP,
            Self::Zero => win::D3D12_STENCIL_OP_ZERO,
            Self::Replace => win::D3D12_STENCIL_OP_REPLACE,
            Self::IncrSat => win::D3D12_STENCIL_OP_INCR_SAT,
            Self::DecrSat => win::D3D12_STENCIL_OP_DECR_SAT,
            Self::Invert => win::D3D12_STENCIL_OP_INVERT,
            Self::Incr => win::D3D12_STENCIL_OP_INCR,
            Self::Decr => win::D3D12_STENCIL_OP_DECR
        }
    }
}

pub struct SDepthStencilOpDesc {
    stencil_fail_op: EStencilOp,
    stencil_depth_fail_op: EStencilOp,
    stencil_pass_op: EStencilOp,
    stencil_func: EComparisonFunc,
}

impl Default for SDepthStencilOpDesc {
    fn default() -> Self {
        Self {
            stencil_fail_op: EStencilOp::Keep,
            stencil_depth_fail_op: EStencilOp::Keep,
            stencil_pass_op: EStencilOp::Keep,
            stencil_func: EComparisonFunc::Always,
        }
    }
}

impl SDepthStencilOpDesc {
    pub fn d3dtype(&self) -> win::D3D12_DEPTH_STENCILOP_DESC {
        win::D3D12_DEPTH_STENCILOP_DESC {
            StencilFailOp: self.stencil_fail_op.d3dtype(),
            StencilDepthFailOp: self.stencil_depth_fail_op.d3dtype(),
            StencilPassOp: self.stencil_depth_fail_op.d3dtype(),
            StencilFunc: self.stencil_func.d3dtype(),
        }
    }
}

pub struct SDepthStencilDesc {
    pub depth_enable: bool,
    pub write_mask: EDepthWriteMask,
    pub depth_func: EComparisonFunc,
    pub stencil_enable: bool,
    pub stencil_read_mask: u8,
    pub stencil_write_mask: u8,
    pub front_face: SDepthStencilOpDesc,
    pub back_face: SDepthStencilOpDesc,
}

impl SDepthStencilDesc {
    pub fn d3dtype(&self) -> win::D3D12_DEPTH_STENCIL_DESC {
        win::D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: win::BOOL::from(self.depth_enable),
            DepthWriteMask: self.write_mask.d3dtype(),
            DepthFunc: self.depth_func.d3dtype(),
            StencilEnable: win::BOOL::from(self.stencil_enable),
            StencilReadMask: self.stencil_read_mask,
            StencilWriteMask: self.stencil_write_mask,
            FrontFace: self.front_face.d3dtype(),
            BackFace: self.back_face.d3dtype(),
        }
    }
}

impl Default for SDepthStencilDesc {
    fn default() -> Self {
        Self {
            depth_enable: true,
            write_mask: EDepthWriteMask::All,
            depth_func: EComparisonFunc::Less,
            stencil_enable: false,
            stencil_read_mask: win::D3D12_DEFAULT_STENCIL_READ_MASK as u8,
            stencil_write_mask: win::D3D12_DEFAULT_STENCIL_READ_MASK as u8,
            front_face: Default::default(),
            back_face: Default::default(),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EPrimitiveTopologyType {
    Undefined,
    Point,
    Line,
    Triangle,
    Patch,
}

impl EPrimitiveTopologyType {
    pub fn d3dtype(&self) -> win::D3D12_PRIMITIVE_TOPOLOGY_TYPE {
        match self {
            Self::Undefined => win::D3D12_PRIMITIVE_TOPOLOGY_TYPE_UNDEFINED,
            Self::Point => win::D3D12_PRIMITIVE_TOPOLOGY_TYPE_POINT,
            Self::Line => win::D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE,
            Self::Triangle => win::D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            Self::Patch => win::D3D12_PRIMITIVE_TOPOLOGY_TYPE_PATCH,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EPrimitiveTopology {
    // -- not comprehensive, too many to type at once, add as needed
    Undefined,
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
    LineListAdj,
    LineStripAdj,
    TriangleListAdj,
    TriangleStripAdj,
}

impl EPrimitiveTopology {
    pub fn d3dtype(&self) -> win::D3D_PRIMITIVE_TOPOLOGY {
        match self {
            Self::Undefined => win::D3D_PRIMITIVE_TOPOLOGY_UNDEFINED,
            Self::PointList => win::D3D_PRIMITIVE_TOPOLOGY_POINTLIST,
            Self::LineList => win::D3D_PRIMITIVE_TOPOLOGY_LINELIST,
            Self::LineStrip => win::D3D_PRIMITIVE_TOPOLOGY_LINESTRIP,
            Self::TriangleList => win::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            Self::TriangleStrip => win::D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
            Self::LineListAdj => win::D3D_PRIMITIVE_TOPOLOGY_LINELIST_ADJ,
            Self::LineStripAdj => win::D3D_PRIMITIVE_TOPOLOGY_LINESTRIP_ADJ,
            Self::TriangleListAdj => win::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST_ADJ,
            Self::TriangleStripAdj => win::D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP_ADJ,
        }
    }
}

#[derive(Clone)]
pub struct SRTFormatArray {
    pub rt_formats: ArrayVec<[EDXGIFormat; 8]>,
}

impl SRTFormatArray {
    pub fn d3dtype(&self) -> win::D3D12_RT_FORMAT_ARRAY {
        unsafe {
            let mut result = mem::MaybeUninit::<win::D3D12_RT_FORMAT_ARRAY>::zeroed();
            (*result.as_mut_ptr()).NumRenderTargets = self.rt_formats.len() as u32;

            for i in 0..self.rt_formats.len() {
                (*result.as_mut_ptr()).RTFormats[i] = self.rt_formats[i].d3dtype();
            }
            for i in self.rt_formats.len()..8 {
                (*result.as_mut_ptr()).RTFormats[i] = EDXGIFormat::Unknown.d3dtype();
            }

            result.assume_init()
        }
    }
}

pub enum EBlend {
    Zero,
    One,
    SrcColor,
    InvSrcColor,
    SrcAlpha,
    InvSrcAlpha,
    DestAlpha,
    InvDestAlpha,
    DestColor,
    InvDestColor,
    SrcAlphaSat,
    BlendFactor,
    InvBlendFactor,
    Src1Color,
    InvSrc1Color,
    Src1Alpha,
    InvSrc1Alpha,
}

impl EBlend {
    pub fn d3dtype(&self) -> win::D3D12_BLEND {
        match self {
            Self::Zero => win::D3D12_BLEND_ZERO,
            Self::One => win::D3D12_BLEND_ONE,
            Self::SrcColor => win::D3D12_BLEND_SRC_COLOR,
            Self::InvSrcColor => win::D3D12_BLEND_INV_SRC_COLOR,
            Self::SrcAlpha => win::D3D12_BLEND_SRC_ALPHA,
            Self::InvSrcAlpha => win::D3D12_BLEND_INV_SRC_ALPHA,
            Self::DestAlpha => win::D3D12_BLEND_DEST_ALPHA,
            Self::InvDestAlpha => win::D3D12_BLEND_INV_DEST_ALPHA,
            Self::DestColor => win::D3D12_BLEND_DEST_COLOR,
            Self::InvDestColor => win::D3D12_BLEND_INV_DEST_COLOR,
            Self::SrcAlphaSat => win::D3D12_BLEND_SRC_ALPHA_SAT,
            Self::BlendFactor => win::D3D12_BLEND_BLEND_FACTOR,
            Self::InvBlendFactor => win::D3D12_BLEND_INV_BLEND_FACTOR,
            Self::Src1Color => win::D3D12_BLEND_SRC1_COLOR,
            Self::InvSrc1Color => win::D3D12_BLEND_INV_SRC1_COLOR,
            Self::Src1Alpha => win::D3D12_BLEND_SRC1_ALPHA,
            Self::InvSrc1Alpha => win::D3D12_BLEND_INV_SRC1_ALPHA
        }
    }
}

pub enum EBlendOp {
    Add,
    Subtract,
    RevSubtract,
    Min,
    Max,
}

impl EBlendOp {
    pub fn d3dtype(&self) -> win::D3D12_BLEND_OP {
        match self {
            Self::Add => win::D3D12_BLEND_OP_ADD,
            Self::Subtract => win::D3D12_BLEND_OP_SUBTRACT,
            Self::RevSubtract => win::D3D12_BLEND_OP_REV_SUBTRACT,
            Self::Min => win::D3D12_BLEND_OP_MIN,
            Self::Max => win::D3D12_BLEND_OP_MAX
        }
    }
}

// -- $$$FRK(FUTURE WORK): consider making this an enum that doesn't allow blend and logic enabled at same time
pub struct SRenderTargetBlendDesc {
    pub blend_enable: bool,
    pub logic_op_enable: bool,
    pub src_blend: EBlend,
    pub dest_blend: EBlend,
    pub blend_op: EBlendOp,
    pub src_blend_alpha: EBlend,
    pub dest_blend_alpha: EBlend,
    pub blend_op_alpha: EBlendOp,
    //logic_op: SLogicOp,
    //render_target_write_mask: u8,
}

impl Default for SRenderTargetBlendDesc {
    fn default() -> Self {
        Self {
            blend_enable: false,
            logic_op_enable: false,
            src_blend: EBlend::One,
            dest_blend: EBlend::Zero,
            blend_op: EBlendOp::Add,
            src_blend_alpha: EBlend::One,
            dest_blend_alpha: EBlend::Zero,
            blend_op_alpha: EBlendOp::Add,
        }
    }
}

impl SRenderTargetBlendDesc {
    pub fn d3dtype(&self) -> win::D3D12_RENDER_TARGET_BLEND_DESC {
        win::D3D12_RENDER_TARGET_BLEND_DESC {
            BlendEnable: win::BOOL::from(self.blend_enable),
            LogicOpEnable: win::BOOL::from(self.logic_op_enable),
            SrcBlend: self.src_blend.d3dtype(),
            DestBlend: self.dest_blend.d3dtype(),
            BlendOp: self.blend_op.d3dtype(),
            SrcBlendAlpha: self.src_blend_alpha.d3dtype(),
            DestBlendAlpha: self.dest_blend_alpha.d3dtype(),
            BlendOpAlpha: self.blend_op_alpha.d3dtype(),
            LogicOp: win::D3D12_LOGIC_OP_NOOP,
            RenderTargetWriteMask: win::D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
        }
    }
}

pub struct SBlendDesc {
    pub alpha_to_coverage_enable: bool,
    pub independent_blend_enable: bool,
    pub render_target_blend_desc: [SRenderTargetBlendDesc; 8],
}

impl Default for SBlendDesc {
    fn default() -> Self {
        Self {
            alpha_to_coverage_enable: false,
            independent_blend_enable: false,
            render_target_blend_desc: [
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
        }
    }
}

impl SBlendDesc {
    pub fn d3dtype(&self) -> win::D3D12_BLEND_DESC {
        let output_render_target =[
            self.render_target_blend_desc[0].d3dtype(),
            self.render_target_blend_desc[1].d3dtype(),
            self.render_target_blend_desc[2].d3dtype(),
            self.render_target_blend_desc[3].d3dtype(),
            self.render_target_blend_desc[4].d3dtype(),
            self.render_target_blend_desc[5].d3dtype(),
            self.render_target_blend_desc[6].d3dtype(),
            self.render_target_blend_desc[7].d3dtype(),
        ];

        win::D3D12_BLEND_DESC {
            AlphaToCoverageEnable: win::BOOL::from(self.alpha_to_coverage_enable),
            IndependentBlendEnable: win::BOOL::from(self.independent_blend_enable),
            RenderTarget: output_render_target,
        }
    }
}

pub enum EFillMode {
    Wireframe,
    Solid,
}

impl EFillMode {
    pub fn d3dtype(&self) -> win::D3D12_FILL_MODE {
        match self {
            Self::Wireframe => win::D3D12_FILL_MODE_WIREFRAME,
            Self::Solid => win::D3D12_FILL_MODE_SOLID,
        }
    }
}

pub enum ECullMode {
    None,
    Front,
    Back,
}

impl ECullMode {
    pub fn d3dtype(&self) -> win::D3D12_CULL_MODE {
        match self {
            Self::None => win::D3D12_CULL_MODE_NONE,
            Self::Front => win::D3D12_CULL_MODE_FRONT,
            Self::Back => win::D3D12_CULL_MODE_BACK,
        }
    }
}

pub enum EConservativeRasterizationMode {
    Off,
    On,
}

impl EConservativeRasterizationMode {
    pub fn d3dtype(&self) -> win::D3D12_CONSERVATIVE_RASTERIZATION_MODE {
        match self {
            Self::Off => win::D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
            Self::On => win::D3D12_CONSERVATIVE_RASTERIZATION_MODE_ON,
        }
    }
}

pub struct SRasterizerDesc {
    fill_mode: EFillMode,
    cull_mode: ECullMode,
    front_counter_clockwise: bool,
    depth_bias: i32,
    depth_bias_clamp: f32,
    slope_scaled_depth_bias: f32,
    depth_clip_enable: bool,
    multisample_enable: bool,
    antialiased_line_enable: bool,
    force_sample_count: u32,
    conservative_raster: EConservativeRasterizationMode,
}

impl Default for SRasterizerDesc {
    fn default() -> Self {
        Self {
            fill_mode: EFillMode::Solid,
            cull_mode: ECullMode::Back,
            front_counter_clockwise: false,
            depth_bias: win::D3D12_DEFAULT_DEPTH_BIAS,
            depth_bias_clamp: win::D3D12_DEFAULT_DEPTH_BIAS_CLAMP,
            slope_scaled_depth_bias: win::D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS,
            depth_clip_enable: true,
            multisample_enable: false,
            antialiased_line_enable: false,
            force_sample_count: 0,
            conservative_raster: EConservativeRasterizationMode::Off,
        }
    }
}

impl SRasterizerDesc {
    pub fn d3dtype(&self) -> win::D3D12_RASTERIZER_DESC {
        win::D3D12_RASTERIZER_DESC {
            FillMode: self.fill_mode.d3dtype(),
            CullMode: self.cull_mode.d3dtype(),
            FrontCounterClockwise: win::BOOL::from(self.front_counter_clockwise),
            DepthBias: self.depth_bias,
            DepthBiasClamp: self.depth_bias_clamp,
            SlopeScaledDepthBias: self.slope_scaled_depth_bias,
            DepthClipEnable: win::BOOL::from(self.depth_clip_enable),
            MultisampleEnable: win::BOOL::from(self.multisample_enable),
            AntialiasedLineEnable: win::BOOL::from(self.antialiased_line_enable),
            ForcedSampleCount: self.force_sample_count,
            ConservativeRaster: self.conservative_raster.d3dtype(),
        }
    }
}

pub struct SSampleDesc {
    count: u32,
    quality: u32,
}

impl Default for SSampleDesc {
    fn default() -> Self {
        Self {
            count: 1,
            quality: 0,
        }
    }
}

impl SSampleDesc {
    pub fn d3dtype(&self) -> win::DXGI_SAMPLE_DESC {
        win::DXGI_SAMPLE_DESC {
            Count: self.count,
            Quality: self.quality,
        }
    }
}

// -- $$$FRK(TODO): skipped a lot of params I don't use
pub struct SGraphicsPipeLineStateDesc<'a> {
    pub root_signature: SRootSignature,
    pub vertex_shader: Option<&'a SShaderBytecode>,
    pub pixel_shader: Option<&'a SShaderBytecode>,
    pub blend_state: SBlendDesc,
    pub sample_mask: u32,
    pub rasterizer_state: SRasterizerDesc,
    pub depth_stencil_state: SDepthStencilDesc,
    pub input_layout: SInputLayoutDesc,
    pub primitive_topology_type: EPrimitiveTopologyType,
    pub num_render_targets: u32,
    pub rtv_formats: Option<SRTFormatArray>,
    pub depth_stencil_format: Option<EDXGIFormat>,
    pub sample_desc: SSampleDesc,
}

impl<'a> SGraphicsPipeLineStateDesc<'a> {
    pub fn new_min(
        root_signature: SRootSignature,
        input_layout: SInputLayoutDesc,
        primitive_topology_type: EPrimitiveTopologyType,
    ) -> Self {
        Self {
            root_signature,
            input_layout,
            primitive_topology_type,

            num_render_targets: 1,
            vertex_shader: None,
            pixel_shader: None,
            blend_state: Default::default(),
            sample_mask: std::u32::MAX,
            depth_stencil_state: Default::default(),
            depth_stencil_format: None,
            rtv_formats: None,
            sample_desc: Default::default(),
            rasterizer_state: SRasterizerDesc::default(),
        }
    }

    // -- unsafe because some of the generated structs cannot outlive self
    pub unsafe fn d3dtype(&mut self) -> win::D3D12_GRAPHICS_PIPELINE_STATE_DESC {
        let mut result = win::D3D12_GRAPHICS_PIPELINE_STATE_DESC::default();

        result.pRootSignature = Some(self.root_signature.raw.clone());
        if let Some(bytecode) = self.vertex_shader {
            result.VS = bytecode.d3dtype();
        }
        if let Some(bytecode) = self.pixel_shader {
            result.PS = bytecode.d3dtype();
        }
        result.BlendState = self.blend_state.d3dtype();
        result.SampleMask = self.sample_mask;
        result.DepthStencilState = self.depth_stencil_state.d3dtype();
        result.InputLayout = self.input_layout.d3dtype();
        result.PrimitiveTopologyType = self.primitive_topology_type.d3dtype();
        result.NumRenderTargets = self.num_render_targets;
        if let Some(depth_stencil_format) = self.depth_stencil_format {
            result.DSVFormat = depth_stencil_format.d3dtype();
        }
        if let Some(rtv_formats) = &self.rtv_formats {
            for (i, format) in rtv_formats.rt_formats.iter().enumerate() {
                result.RTVFormats[i] = format.d3dtype();
            }
        }
        result.SampleDesc = self.sample_desc.d3dtype();
        result.RasterizerState = self.rasterizer_state.d3dtype();

        result
    }
}

// -- $$$FRK(TODO): skipped a lot of params I don't use
pub struct SComputePipelineStateDesc<'a> {
    pub root_signature: SRootSignature,
    pub compute_shader: &'a SShaderBytecode,
}

impl<'a> SComputePipelineStateDesc<'a> {
    // -- unsafe because some of the generated structs cannot outlive this
    pub unsafe fn d3dtype(&self) -> win::D3D12_COMPUTE_PIPELINE_STATE_DESC {
        let mut result = win::D3D12_COMPUTE_PIPELINE_STATE_DESC::default();

        result.pRootSignature = Some(self.root_signature.raw.clone());
        result.CS = self.compute_shader.d3dtype();

        result
    }
}

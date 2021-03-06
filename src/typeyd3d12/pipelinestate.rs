use super::*;

#[derive(Copy, Clone, PartialEq)]
pub enum EInputClassification {
    PerVertexData,
    PerInstanceData,
}

impl EInputClassification {
    pub fn d3dtype(&self) -> D3D12_INPUT_CLASSIFICATION {
        match self {
            Self::PerVertexData => D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            Self::PerInstanceData => D3D12_INPUT_CLASSIFICATION_PER_INSTANCE_DATA,
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

    semantic_name_null_terminated: [winapi::um::winnt::CHAR; 32],
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
            result.semantic_name_null_terminated[i] = *c as i8;
            i += 1;
        }
        result.semantic_name_null_terminated[i] = 0;

        result
    }

    pub unsafe fn d3dtype(&self) -> D3D12_INPUT_ELEMENT_DESC {
        D3D12_INPUT_ELEMENT_DESC {
            //SemanticName: self.semantic_name_utf16.as_ptr(),
            SemanticName: self.semantic_name_null_terminated.as_ptr(),
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
    raw: ComPtr<ID3D12PipelineState>,
}

impl SPipelineState {
    pub unsafe fn new_from_raw(raw: ComPtr<ID3D12PipelineState>) -> Self {
        Self { raw: raw }
    }

    pub unsafe fn raw(&self) -> &ComPtr<ID3D12PipelineState> {
        &self.raw
    }
}

pub struct SInputLayoutDesc {
    pub input_element_descs: ArrayVec<[SInputElementDesc; 16]>,

    pub d3d_input_element_descs: ArrayVec<[D3D12_INPUT_ELEMENT_DESC; 16]>,
}

impl SInputLayoutDesc {
    pub unsafe fn generate_d3dtype(&mut self) {
        self.d3d_input_element_descs.clear();

        for input_element_desc in &self.input_element_descs {
            self.d3d_input_element_descs
                .push(input_element_desc.d3dtype());
        }
    }

    pub unsafe fn d3dtype(&mut self) -> D3D12_INPUT_LAYOUT_DESC {
        // -- $$$FRK(NOTE): the generate data here is no longer valid if this moves!!!
        // -- it contains internal references!
        self.generate_d3dtype();

        let result = D3D12_INPUT_LAYOUT_DESC {
            pInputElementDescs: self.d3d_input_element_descs.as_ptr(),
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
    pub fn d3dtype(&self) -> D3D12_STENCIL_OP {
        match self {
            Self::Keep => D3D12_STENCIL_OP_KEEP,
            Self::Zero => D3D12_STENCIL_OP_ZERO,
            Self::Replace => D3D12_STENCIL_OP_REPLACE,
            Self::IncrSat => D3D12_STENCIL_OP_INCR_SAT,
            Self::DecrSat => D3D12_STENCIL_OP_DECR_SAT,
            Self::Invert => D3D12_STENCIL_OP_INVERT,
            Self::Incr => D3D12_STENCIL_OP_INCR,
            Self::Decr => D3D12_STENCIL_OP_DECR
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
    pub fn d3dtype(&self) -> D3D12_DEPTH_STENCILOP_DESC {
        D3D12_DEPTH_STENCILOP_DESC {
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
    pub fn d3dtype(&self) -> D3D12_DEPTH_STENCIL_DESC {
        D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: if self.depth_enable { 1 } else { 0 } ,
            DepthWriteMask: self.write_mask.d3dtype(),
            DepthFunc: self.depth_func.d3dtype(),
            StencilEnable: if self.stencil_enable { 1 } else { 0 } ,
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
            stencil_read_mask: D3D12_DEFAULT_STENCIL_READ_MASK as u8,
            stencil_write_mask: D3D12_DEFAULT_STENCIL_READ_MASK as u8,
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
    pub fn d3dtype(&self) -> D3D12_PRIMITIVE_TOPOLOGY_TYPE {
        match self {
            Self::Undefined => D3D12_PRIMITIVE_TOPOLOGY_TYPE_UNDEFINED,
            Self::Point => D3D12_PRIMITIVE_TOPOLOGY_TYPE_POINT,
            Self::Line => D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE,
            Self::Triangle => D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            Self::Patch => D3D12_PRIMITIVE_TOPOLOGY_TYPE_PATCH,
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
    pub fn d3dtype(&self) -> D3D12_PRIMITIVE_TOPOLOGY {
        match self {
            Self::Undefined => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_UNDEFINED,
            Self::PointList => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_POINTLIST,
            Self::LineList => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_LINELIST,
            Self::LineStrip => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_LINESTRIP,
            Self::TriangleList => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            Self::TriangleStrip => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
            Self::LineListAdj => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_LINELIST_ADJ,
            Self::LineStripAdj => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_LINESTRIP_ADJ,
            Self::TriangleListAdj => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST_ADJ,
            Self::TriangleStripAdj => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP_ADJ,
        }
    }
}

pub struct SRTFormatArray {
    pub rt_formats: ArrayVec<[EDXGIFormat; 8]>,
}

impl SRTFormatArray {
    pub fn d3dtype(&self) -> D3D12_RT_FORMAT_ARRAY {
        unsafe {
            let mut result = mem::MaybeUninit::<D3D12_RT_FORMAT_ARRAY>::zeroed();
            (*result.as_mut_ptr()).NumRenderTargets = self.rt_formats.len() as UINT;

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
    pub fn d3dtype(&self) -> D3D12_BLEND {
        match self {
            Self::Zero => D3D12_BLEND_ZERO,
            Self::One => D3D12_BLEND_ONE,
            Self::SrcColor => D3D12_BLEND_SRC_COLOR,
            Self::InvSrcColor => D3D12_BLEND_INV_SRC_COLOR,
            Self::SrcAlpha => D3D12_BLEND_SRC_ALPHA,
            Self::InvSrcAlpha => D3D12_BLEND_INV_SRC_ALPHA,
            Self::DestAlpha => D3D12_BLEND_DEST_ALPHA,
            Self::InvDestAlpha => D3D12_BLEND_INV_DEST_ALPHA,
            Self::DestColor => D3D12_BLEND_DEST_COLOR,
            Self::InvDestColor => D3D12_BLEND_INV_DEST_COLOR,
            Self::SrcAlphaSat => D3D12_BLEND_SRC_ALPHA_SAT,
            Self::BlendFactor => D3D12_BLEND_BLEND_FACTOR,
            Self::InvBlendFactor => D3D12_BLEND_INV_BLEND_FACTOR,
            Self::Src1Color => D3D12_BLEND_SRC1_COLOR,
            Self::InvSrc1Color => D3D12_BLEND_INV_SRC1_COLOR,
            Self::Src1Alpha => D3D12_BLEND_SRC1_ALPHA,
            Self::InvSrc1Alpha => D3D12_BLEND_INV_SRC1_ALPHA
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
    pub fn d3dtype(&self) -> D3D12_BLEND {
        match self {
            Self::Add => D3D12_BLEND_OP_ADD,
            Self::Subtract => D3D12_BLEND_OP_SUBTRACT,
            Self::RevSubtract => D3D12_BLEND_OP_REV_SUBTRACT,
            Self::Min => D3D12_BLEND_OP_MIN,
            Self::Max => D3D12_BLEND_OP_MAX
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
    pub fn d3dtype(&self) -> D3D12_RENDER_TARGET_BLEND_DESC {
        D3D12_RENDER_TARGET_BLEND_DESC {
            BlendEnable: self.blend_enable as i32,
            LogicOpEnable: self.logic_op_enable as i32,
            SrcBlend: self.src_blend.d3dtype(),
            DestBlend: self.dest_blend.d3dtype(),
            BlendOp: self.blend_op.d3dtype(),
            SrcBlendAlpha: self.src_blend_alpha.d3dtype(),
            DestBlendAlpha: self.dest_blend_alpha.d3dtype(),
            BlendOpAlpha: self.blend_op_alpha.d3dtype(),
            LogicOp: D3D12_LOGIC_OP_NOOP,
            RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL as u8,
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
    pub fn d3dtype(&self) -> D3D12_BLEND_DESC {
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

        D3D12_BLEND_DESC {
            AlphaToCoverageEnable: self.alpha_to_coverage_enable as i32,
            IndependentBlendEnable: self.independent_blend_enable as i32,
            RenderTarget: output_render_target,
        }
    }
}

pub struct SPipelineStateStreamDesc<'a, T> {
    stream: &'a T,
}

impl<'a, T> SPipelineStateStreamDesc<'a, T> {
    pub fn create(stream: &'a T) -> Self {
        Self { stream: stream }
    }

    pub unsafe fn d3dtype(&self) -> D3D12_PIPELINE_STATE_STREAM_DESC {
        D3D12_PIPELINE_STATE_STREAM_DESC {
            SizeInBytes: mem::size_of::<T>() as winapi::shared::basetsd::SIZE_T,
            pPipelineStateSubobjectStream: self.stream as *const T as *mut c_void,
        }
    }
}

pub enum EPipelineStateSubobjectType {
    RootSignature,
    VS,
    PS,
    DS,
    HS,
    GS,
    CS,
    StreamOutput,
    Blend,
    SampleMask,
    Rasterizer,
    DepthStencil,
    InputLayout,
    IBStripCutValue,
    PrimitiveTopology,
    RenderTargetFormats,
    DepthStencilFormat,
    SampleDesc,
    NodeMask,
    CachedPSO,
    Flags,
    DepthStencil1,
    //ViewInstancing,
    MaxValid,
}

impl EPipelineStateSubobjectType {
    pub fn d3dtype(&self) -> D3D12_PIPELINE_STATE_SUBOBJECT_TYPE {
        match self {
            Self::RootSignature => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_ROOT_SIGNATURE,
            Self::VS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_VS,
            Self::PS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PS,
            Self::DS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DS,
            Self::HS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_HS,
            Self::GS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_GS,
            Self::CS => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_CS,
            Self::StreamOutput => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_STREAM_OUTPUT,
            Self::Blend => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_BLEND,
            Self::SampleMask => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_MASK,
            Self::Rasterizer => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RASTERIZER,
            Self::DepthStencil => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL,
            Self::InputLayout => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_INPUT_LAYOUT,
            Self::IBStripCutValue => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_IB_STRIP_CUT_VALUE,
            Self::PrimitiveTopology => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PRIMITIVE_TOPOLOGY,
            Self::RenderTargetFormats => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RENDER_TARGET_FORMATS,
            Self::DepthStencilFormat => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL_FORMAT,
            Self::SampleDesc => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_DESC,
            Self::NodeMask => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_NODE_MASK,
            Self::CachedPSO => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_CACHED_PSO,
            Self::Flags => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_FLAGS,
            Self::DepthStencil1 => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL1,
            //Self::ViewInstancing => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_VIEW_INSTANCING,
            Self::MaxValid => D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_MAX_VALID,
        }
    }
}

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
    semantic_name: &'static str,
    semantic_index: u32,
    format: EDXGIFormat,
    input_slot: u32,
    aligned_byte_offset: u32,
    input_slot_class: EInputClassification,
    instance_data_step_rate: u32,

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
    input_element_descs: ArrayVec<[SInputElementDesc; 16]>,

    d3d_input_element_descs: ArrayVec<[D3D12_INPUT_ELEMENT_DESC; 16]>,
}

impl SInputLayoutDesc {
    // -- $$$FRK(TODO): This probably belongs in niced3d12
    pub fn create(input_element_descs: &[SInputElementDesc]) -> Self {
        let mut result = Self {
            input_element_descs: ArrayVec::new(),
            d3d_input_element_descs: ArrayVec::new(),
        };

        result
            .input_element_descs
            .try_extend_from_slice(input_element_descs)
            .unwrap();
        result
    }

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
    depth_enable: bool,
    write_mask: EDepthWriteMask,
    depth_func: EComparisonFunc,
    stencil_enable: bool,
    stencil_read_mask: u8,
    stencil_write_mask: u8,
    front_face: SDepthStencilOpDesc,
    back_face: SDepthStencilOpDesc,
}

impl SDepthStencilDesc {
    pub fn d3dtype(&self) -> D3D12_DEPTH_STENCIL_DESC {
        D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: self.depth_enable,
            DepthWriteMask: self.write_mask.d3dtype(),
            DepthFunc: self.depth_func.d3dtype(),
            StencilEnable: self.stencil_enable,
            StencilReadMask: self.stencil_read_mask,
            StencilWriteMask: self.stencil_write_mask,
            FrontFace: self.front_face.d3dtype(),
            BackFace: self.back_face.d3dtype(),
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
    TriangleList,
}

impl EPrimitiveTopology {
    pub fn d3dtype(&self) -> D3D12_PRIMITIVE_TOPOLOGY {
        match self {
            Self::TriangleList => d3dcommon::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
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

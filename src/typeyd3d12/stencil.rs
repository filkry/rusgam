use super::*;

#[derive(Copy, Clone, PartialEq)]
enum EStencilOp {
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
            Self::Decr => D3D12_STENCIL_OP_DECR,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
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
            StencilPassOp: self.stencil_pass_op.d3dtype(),
            StencilFunc: self.stencil_func.d3dtype(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct SDepthStencilDesc {
    depth_enable: bool,
    depth_write_mask: EDepthWriteMask,
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
            DepthEnable: self.depth_enable as BOOL,
            DepthWriteMask: self.depth_write_mask.d3dtype(),
            DepthFunc: self.depth_func.d3dtype(),
            StencilEnable: self.stencil_enable as BOOL,
            StencilReadMask: self.stencil_read_mask,
            StencilWriteMask: self.stencil_write_mask,
            FrontFace: self.front_face.d3dtype(),
            BackFace: self.back_face.d3dtype(),
        }
    }
}



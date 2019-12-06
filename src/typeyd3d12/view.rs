use super::*;

pub struct SVertexBufferView {
    raw: D3D12_VERTEX_BUFFER_VIEW,
}

impl SVertexBufferView {
    pub fn create(
        bufferlocation: SGPUVirtualAddress,
        sizeinbytes: u32,
        strideinbytes: u32,
    ) -> Self {
        Self {
            raw: D3D12_VERTEX_BUFFER_VIEW {
                BufferLocation: bufferlocation.raw(),
                SizeInBytes: sizeinbytes,
                StrideInBytes: strideinbytes,
            },
        }
    }

    pub unsafe fn raw(&self) -> &D3D12_VERTEX_BUFFER_VIEW {
        &self.raw
    }
}

pub struct SIndexBufferView {
    raw: D3D12_INDEX_BUFFER_VIEW,
}

impl SIndexBufferView {
    pub fn create(bufferlocation: SGPUVirtualAddress, format: EDXGIFormat, sizeinbytes: u32) -> Self {
        Self {
            raw: D3D12_INDEX_BUFFER_VIEW {
                BufferLocation: bufferlocation.raw(),
                Format: format.d3dtype(),
                SizeInBytes: sizeinbytes,
            },
        }
    }

    pub unsafe fn raw(&self) -> &D3D12_INDEX_BUFFER_VIEW {
        &self.raw
    }
}

pub enum EDepthStencilViewDescData {
    Tex2D(STex2DDSV),
}

pub struct SDepthStencilViewDesc {
    pub format: EDXGIFormat,
    pub view_dimension: EDSVDimension,
    pub flags: SDSVFlags,
    pub data: EDepthStencilViewDescData,
}

impl SDepthStencilViewDesc {
    pub fn d3dtype(&self) -> D3D12_DEPTH_STENCIL_VIEW_DESC {
        unsafe {
            let mut result : D3D12_DEPTH_STENCIL_VIEW_DESC = mem::uninitialized();
            result.Format = self.format.d3dtype();
            result.ViewDimension = self.view_dimension.d3dtype();
            result.Flags = self.flags.d3dtype();

            match &self.data {
                EDepthStencilViewDescData::Tex2D(tex2d_dsv) => *(result.u.Texture2D_mut()) = tex2d_dsv.d3dtype(),
            }

            result
        }
    }
}

pub struct SDepthStencilValue {
    pub depth: f32,
    pub stencil: u8,
}

impl SDepthStencilValue {
    pub fn d3dtype(&self) -> D3D12_DEPTH_STENCIL_VALUE {
        D3D12_DEPTH_STENCIL_VALUE {
            Depth: self.depth,
            Stencil: self.stencil,
        }
    }
}

pub enum EClearValue {
    Color([f32; 4]),
    DepthStencil(SDepthStencilValue),
}

pub struct SClearValue {
    pub format: EDXGIFormat,
    pub value: EClearValue,
}

impl SClearValue {
    pub fn d3dtype(&self) -> D3D12_CLEAR_VALUE {
        unsafe {
            let mut result : D3D12_CLEAR_VALUE = mem::uninitialized();
            result.Format = self.format.d3dtype();
            match &self.value {
                EClearValue::Color(color) => *(result.u.Color_mut()) = *color,
                EClearValue::DepthStencil(depth_stencil_value) => *(result.u.DepthStencil_mut()) = depth_stencil_value.d3dtype(),
            }
            result
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EDSVDimension {
    Unknown,
    Texture1D,
    Texture1DArray,
    Texture2D,
    Texture2DArray,
    Texture2DMS,
    Texture2DMSArray,
}

impl EDSVDimension {
    pub fn d3dtype(&self) -> D3D12_DSV_DIMENSION {
        match self {
            Self::Unknown => D3D12_DSV_DIMENSION_UNKNOWN,
            Self::Texture1D => D3D12_DSV_DIMENSION_TEXTURE1D,
            Self::Texture1DArray => D3D12_DSV_DIMENSION_TEXTURE1DARRAY,
            Self::Texture2D => D3D12_DSV_DIMENSION_TEXTURE2D,
            Self::Texture2DArray => D3D12_DSV_DIMENSION_TEXTURE2DARRAY,
            Self::Texture2DMS => D3D12_DSV_DIMENSION_TEXTURE2DMS,
            Self::Texture2DMSArray => D3D12_DSV_DIMENSION_TEXTURE2DMSARRAY
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EDSVFlags {
    None,
    ReadOnlyDepth,
    ReadOnlyStencil,
}

impl TD3DFlags32 for EDSVFlags {
    type TD3DType = D3D12_DSV_FLAGS;

    fn d3dtype(&self) -> Self::TD3DType {
        match self {
            Self::None => D3D12_DSV_FLAG_NONE,
            Self::ReadOnlyDepth => D3D12_DSV_FLAG_READ_ONLY_DEPTH,
            Self::ReadOnlyStencil => D3D12_DSV_FLAG_READ_ONLY_STENCIL
        }
    }
}
pub type SDSVFlags = SD3DFlags32<EDSVFlags>;

pub struct STex2DDSV {
    pub mip_slice: u32,
}

impl STex2DDSV {
    pub fn d3dtype(&self) -> D3D12_TEX2D_DSV {
        D3D12_TEX2D_DSV {
            MipSlice: self.mip_slice,
        }
    }
}

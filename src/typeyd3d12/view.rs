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
    pub fn create(
        bufferlocation: SGPUVirtualAddress,
        format: EDXGIFormat,
        sizeinbytes: u32,
    ) -> Self {
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
            let mut result = mem::MaybeUninit::<D3D12_DEPTH_STENCIL_VIEW_DESC>::zeroed();
            (*result.as_mut_ptr()).Format = self.format.d3dtype();
            (*result.as_mut_ptr()).ViewDimension = self.view_dimension.d3dtype();
            (*result.as_mut_ptr()).Flags = self.flags.d3dtype();

            match &self.data {
                EDepthStencilViewDescData::Tex2D(tex2d_dsv) => {
                    *((*result.as_mut_ptr()).u.Texture2D_mut()) = tex2d_dsv.d3dtype()
                }
            }

            result.assume_init()
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
            let mut result = mem::MaybeUninit::<D3D12_CLEAR_VALUE>::zeroed();
            (*result.as_mut_ptr()).Format = self.format.d3dtype();
            match &self.value {
                EClearValue::Color(color) => *((*result.as_mut_ptr()).u.Color_mut()) = *color,
                EClearValue::DepthStencil(depth_stencil_value) => {
                    *((*result.as_mut_ptr()).u.DepthStencil_mut()) = depth_stencil_value.d3dtype()
                }
            }
            result.assume_init()
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
            Self::Texture2DMSArray => D3D12_DSV_DIMENSION_TEXTURE2DMSARRAY,
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
            Self::ReadOnlyStencil => D3D12_DSV_FLAG_READ_ONLY_STENCIL,
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

pub struct STex2DSRV {
    pub most_detailed_mip: u32,
    pub mip_levels: u32,
    pub plane_slice: u32,
    pub resource_min_lod_clamp: f32,
}

impl Default for STex2DSRV {

    fn default() -> Self {
        STex2DSRV {
            most_detailed_mip: 0,
            mip_levels: 0,
            plane_slice: 0,
            resource_min_lod_clamp: 0.0,
        }
    }
}

impl STex2DSRV {
    pub fn d3dtype(&self) -> D3D12_TEX2D_SRV {
        D3D12_TEX2D_SRV {
            MostDetailedMip: self.most_detailed_mip,
            MipLevels: self.mip_levels,
            PlaneSlice: self.plane_slice,
            ResourceMinLODClamp: self.resource_min_lod_clamp,
        }
    }
}

pub enum ESRV {
    Texture2D {
        data: STex2DSRV,
    }
}

impl ESRV {
    pub fn d3d_view_dimension(&self) -> D3D12_SRV_DIMENSION {
        match self {
            Self::Texture2D{..} => D3D12_SRV_DIMENSION_TEXTURE2D,
        }
    }
}

pub struct SShaderResourceViewDesc {
    pub format: EDXGIFormat,
    pub view: ESRV, // combines view_dimension with the underlying data
    //shader_4_component_mapping: u32, $$$FRK(TODO): only support default currently
}

impl SShaderResourceViewDesc {
    pub fn d3dtype(&self) -> D3D12_SHADER_RESOURCE_VIEW_DESC {
        unsafe {
            let mut result = mem::MaybeUninit::<D3D12_SHADER_RESOURCE_VIEW_DESC>::zeroed();
            (*result.as_mut_ptr()).Format = self.format.d3dtype();
            (*result.as_mut_ptr()).ViewDimension = self.view.d3d_view_dimension();
            match &self.view {
                ESRV::Texture2D{data} => {
                    *(*result.as_mut_ptr()).u.Texture2D_mut() = data.d3dtype();
                }
            }

            // -- recreating D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING
            let mut mapping : u32 = 0;
            let bit_shift = 3;
            mapping = mapping | (0 << 0);
            mapping = mapping | (1 << bit_shift);
            mapping = mapping | (2 << (bit_shift * 2));
            mapping = mapping | (3 << (bit_shift * 3));
            mapping = mapping | (1 << (bit_shift * 4)); // D3D12_SHADER_COMPONENT_MAPPING_ALWAYS_SET_BIT_AVOIDING_ZEROMEM_MISTAKES

            (*result.as_mut_ptr()).Shader4ComponentMapping = mapping;
            result.assume_init()
        }
    }
}
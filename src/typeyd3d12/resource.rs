use super::*;

#[derive(Copy, Clone, PartialEq)]
pub enum EResourceStates {
    Common,
    VertexAndConstantBuffer,
    IndexBuffer,
    RenderTarget,
    UnorderedAccess,
    DepthWrite,
    DepthRead,
    NonPixelShaderResource,
    PixelShaderResource,
    StreamOut,
    IndirectArgument,
    CopyDest,
    CopySource,
    ResolveDest,
    ResolveSource,
    GenericRead,
    Present,
    Predication,
}

impl EResourceStates {
    pub fn d3dtype(&self) -> D3D12_RESOURCE_STATES {
        match self {
            EResourceStates::Common => D3D12_RESOURCE_STATE_COMMON,
            EResourceStates::VertexAndConstantBuffer => {
                D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER
            }
            EResourceStates::IndexBuffer => D3D12_RESOURCE_STATE_INDEX_BUFFER,
            EResourceStates::RenderTarget => D3D12_RESOURCE_STATE_RENDER_TARGET,
            EResourceStates::UnorderedAccess => D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            EResourceStates::DepthWrite => D3D12_RESOURCE_STATE_DEPTH_WRITE,
            EResourceStates::DepthRead => D3D12_RESOURCE_STATE_DEPTH_READ,
            EResourceStates::NonPixelShaderResource => {
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE
            }
            EResourceStates::PixelShaderResource => D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
            EResourceStates::StreamOut => D3D12_RESOURCE_STATE_STREAM_OUT,
            EResourceStates::IndirectArgument => D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
            EResourceStates::CopyDest => D3D12_RESOURCE_STATE_COPY_DEST,
            EResourceStates::CopySource => D3D12_RESOURCE_STATE_COPY_SOURCE,
            EResourceStates::ResolveDest => D3D12_RESOURCE_STATE_RESOLVE_DEST,
            EResourceStates::ResolveSource => D3D12_RESOURCE_STATE_RESOLVE_SOURCE,
            EResourceStates::GenericRead => D3D12_RESOURCE_STATE_GENERIC_READ,
            EResourceStates::Present => D3D12_RESOURCE_STATE_PRESENT,
            EResourceStates::Predication => D3D12_RESOURCE_STATE_PREDICATION,
        }
    }
}

pub struct SResourceDesc {
    raw: D3D12_RESOURCE_DESC,
}

// -- $$$FRK(TODO): does not follow the philosophy of this file for creating rustic types for each
// -- D3D type. Furthermore, the helper methods belong in niced3d12
impl SResourceDesc {
    pub unsafe fn raw(&self) -> &D3D12_RESOURCE_DESC {
        &self.raw
    }

    pub fn createbuffer(buffersize: usize, flags: SResourceFlags) -> Self {
        Self {
            raw: D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                Alignment: D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT as u64,
                Width: buffersize as u64, // seems like this is used as the main dimension for a 1D resource
                Height: 1,                // required
                DepthOrArraySize: 1,      // required
                MipLevels: 1,             // required
                Format: dxgiformat::DXGI_FORMAT_UNKNOWN, // required
                SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                    Count: 1,   // required
                    Quality: 0, // required
                },
                Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR, // required
                Flags: flags.rawtype(),
            },
        }
    }

    pub fn create_texture_2d(
        width: u32,
        height: u32,
        array_size: u16,
        mip_levels: u16,
        format: EDXGIFormat,
        flags: SResourceFlags,
    ) -> Self {
        Self {
            raw: D3D12_RESOURCE_DESC {
                Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Alignment: D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT as u64,
                Width: width as u64,
                Height: height,               // required
                DepthOrArraySize: array_size, // required
                MipLevels: mip_levels,        // required
                Format: format.d3dtype(),     // required
                SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                    Count: 1,   // required
                    Quality: 0, // required
                },
                Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN, // required
                Flags: flags.rawtype(),
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EResourceFlags {
    ENone,
    AllowRenderTarget,
    AllowDepthStencil,
    AllowUnorderedAccess,
    DenyShaderResource,
    AllowCrossAdapter,
    AllowSimultaneousAccess,
}

impl TEnumFlags32 for EResourceFlags {
    type TRawType = D3D12_HEAP_FLAGS;

    fn rawtype(&self) -> Self::TRawType {
        match self {
            EResourceFlags::ENone => D3D12_RESOURCE_FLAG_NONE,
            EResourceFlags::AllowRenderTarget => D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
            EResourceFlags::AllowDepthStencil => D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
            EResourceFlags::AllowUnorderedAccess => D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            EResourceFlags::DenyShaderResource => D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE,
            EResourceFlags::AllowCrossAdapter => D3D12_RESOURCE_FLAG_ALLOW_CROSS_ADAPTER,
            EResourceFlags::AllowSimultaneousAccess => {
                D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS
            }
        }
    }
}

pub type SResourceFlags = SEnumFlags32<EResourceFlags>;

#[derive(Clone)]
pub struct SResource {
    resource: ComPtr<ID3D12Resource>,
}

impl std::cmp::PartialEq for SResource {
    fn eq(&self, other: &Self) -> bool {
        self.resource == other.resource
    }
}

impl SResource {
    pub unsafe fn new_from_raw(raw: ComPtr<ID3D12Resource>) -> Self {
        Self { resource: raw }
    }

    pub unsafe fn raw(&self) -> &ComPtr<ID3D12Resource> {
        &self.resource
    }

    pub unsafe fn raw_mut(&mut self) -> &mut ComPtr<ID3D12Resource> {
        &mut self.resource
    }

    pub fn getgpuvirtualaddress(&self) -> SGPUVirtualAddress {
        unsafe {
            SGPUVirtualAddress {
                raw: self.resource.GetGPUVirtualAddress(),
            }
        }
    }

    pub unsafe fn map(
        &self,
        subresource: u32,
        read_range: Option<SRange>,
    ) -> Result<*mut u8, &'static str> {
        let read_range_d3d = read_range.map(|r| r.d3dtype());
        let read_range_d3d_ptr = read_range_d3d.as_ref().map_or(std::ptr::null(), |r| r);

        let mut raw_result = std::ptr::null_mut() as *mut c_void;
        let hr = self
            .resource
            .Map(subresource, read_range_d3d_ptr, &mut raw_result);
        returnerrifwinerror!(hr, "Failed to map subresource.");

        Ok(raw_result as *mut u8)
    }
}

pub fn create_transition_barrier(
    resource: &SResource,
    beforestate: EResourceStates,
    afterstate: EResourceStates,
) -> SBarrier {
    let mut barrier = D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        u: unsafe { mem::zeroed() },
    };

    *unsafe { barrier.u.Transition_mut() } = D3D12_RESOURCE_TRANSITION_BARRIER {
        pResource: resource.resource.as_raw(),
        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
        StateBefore: beforestate.d3dtype(),
        StateAfter: afterstate.d3dtype(),
    };

    SBarrier { barrier: barrier }
}

pub struct SSubResourceData {
    raw: D3D12_SUBRESOURCE_DATA,
}

impl SSubResourceData {
    pub unsafe fn create<T>(data: *const T, rowpitch: usize, slicepitch: usize) -> Self {
        let subresourcedata = D3D12_SUBRESOURCE_DATA {
            pData: data as *const c_void,
            RowPitch: rowpitch as isize,
            SlicePitch: slicepitch as isize,
        };
        SSubResourceData {
            raw: subresourcedata,
        }
    }

    pub unsafe fn raw_mut(&mut self) -> &mut D3D12_SUBRESOURCE_DATA {
        &mut self.raw
    }
}

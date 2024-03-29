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
    pub fn d3dtype(&self) -> win::D3D12_RESOURCE_STATES {
        match self {
            EResourceStates::Common => win::D3D12_RESOURCE_STATE_COMMON,
            EResourceStates::VertexAndConstantBuffer => {
                win::D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER
            }
            EResourceStates::IndexBuffer => win::D3D12_RESOURCE_STATE_INDEX_BUFFER,
            EResourceStates::RenderTarget => win::D3D12_RESOURCE_STATE_RENDER_TARGET,
            EResourceStates::UnorderedAccess => win::D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            EResourceStates::DepthWrite => win::D3D12_RESOURCE_STATE_DEPTH_WRITE,
            EResourceStates::DepthRead => win::D3D12_RESOURCE_STATE_DEPTH_READ,
            EResourceStates::NonPixelShaderResource => {
                win::D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE
            }
            EResourceStates::PixelShaderResource => win::D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
            EResourceStates::StreamOut => win::D3D12_RESOURCE_STATE_STREAM_OUT,
            EResourceStates::IndirectArgument => win::D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
            EResourceStates::CopyDest => win::D3D12_RESOURCE_STATE_COPY_DEST,
            EResourceStates::CopySource => win::D3D12_RESOURCE_STATE_COPY_SOURCE,
            EResourceStates::ResolveDest => win::D3D12_RESOURCE_STATE_RESOLVE_DEST,
            EResourceStates::ResolveSource => win::D3D12_RESOURCE_STATE_RESOLVE_SOURCE,
            EResourceStates::GenericRead => win::D3D12_RESOURCE_STATE_GENERIC_READ,
            EResourceStates::Present => win::D3D12_RESOURCE_STATE_PRESENT,
            EResourceStates::Predication => win::D3D12_RESOURCE_STATE_PREDICATION,
        }
    }
}

pub struct SResourceDesc {
    raw: win::D3D12_RESOURCE_DESC,
}

// -- $$$FRK(FUTURE WORK): does not follow the philosophy of this file for creating rustic types for each
// -- D3D type. Furthermore, the helper methods belong in niced3d12
impl SResourceDesc {
    pub unsafe fn raw(&self) -> &win::D3D12_RESOURCE_DESC {
        &self.raw
    }

    pub fn createbuffer(buffersize: usize, flags: SResourceFlags) -> Self {
        Self {
            raw: win::D3D12_RESOURCE_DESC {
                Dimension: win::D3D12_RESOURCE_DIMENSION_BUFFER,
                Alignment: win::D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT as u64,
                Width: buffersize as u64, // seems like this is used as the main dimension for a 1D resource
                Height: 1,                // required
                DepthOrArraySize: 1,      // required
                MipLevels: 1,             // required
                Format: win::DXGI_FORMAT_UNKNOWN, // required
                SampleDesc: win::DXGI_SAMPLE_DESC {
                    Count: 1,   // required
                    Quality: 0, // required
                },
                Layout: win::D3D12_TEXTURE_LAYOUT_ROW_MAJOR, // required
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
            raw: win::D3D12_RESOURCE_DESC {
                Dimension: win::D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                Alignment: win::D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT as u64,
                Width: width as u64,
                Height: height,               // required
                DepthOrArraySize: array_size, // required
                MipLevels: mip_levels,        // required
                Format: format.d3dtype(),     // required
                SampleDesc: win::DXGI_SAMPLE_DESC {
                    Count: 1,   // required
                    Quality: 0, // required
                },
                Layout: win::D3D12_TEXTURE_LAYOUT_UNKNOWN, // required
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

impl TEnumFlags for EResourceFlags {
    type TRawType = win::D3D12_RESOURCE_FLAGS;

    fn rawtype(&self) -> Self::TRawType {
        match self {
            EResourceFlags::ENone => win::D3D12_RESOURCE_FLAG_NONE,
            EResourceFlags::AllowRenderTarget => win::D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
            EResourceFlags::AllowDepthStencil => win::D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL,
            EResourceFlags::AllowUnorderedAccess => win::D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            EResourceFlags::DenyShaderResource => win::D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE,
            EResourceFlags::AllowCrossAdapter => win::D3D12_RESOURCE_FLAG_ALLOW_CROSS_ADAPTER,
            EResourceFlags::AllowSimultaneousAccess => {
                win::D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS
            }
        }
    }
}

pub type SResourceFlags = SEnumFlags<EResourceFlags>;

#[derive(Clone)]
pub struct SResource {
    resource: win::ID3D12Resource,
}

impl std::cmp::PartialEq for SResource {
    fn eq(&self, other: &Self) -> bool {
        self.resource == other.resource
    }
}

impl SResource {
    pub unsafe fn new_from_raw(raw: win::ID3D12Resource) -> Self {
        Self { resource: raw }
    }

    pub unsafe fn raw(&self) -> &win::ID3D12Resource {
        &self.resource
    }

    pub unsafe fn raw_mut(&mut self) -> &mut win::ID3D12Resource {
        &mut self.resource
    }

    pub fn get_gpu_virtual_address(&self) -> SGPUVirtualAddress {
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

        let mut raw_result = std::ptr::null_mut() as *mut std::ffi::c_void;
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
    let mut barrier = win::D3D12_RESOURCE_BARRIER {
        Type: win::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: win::D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: unsafe { mem::zeroed() },
    };

    use crate::win::Abi;
    barrier.Anonymous.Transition = win::D3D12_RESOURCE_TRANSITION_BARRIER {
        pResource: Some(resource.resource.clone()),
        Subresource: win::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
        StateBefore: beforestate.d3dtype(),
        StateAfter: afterstate.d3dtype(),
    }.abi();

    SBarrier { barrier: barrier }
}

pub struct SSubResourceData {
    raw: win::D3D12_SUBRESOURCE_DATA,
}

impl SSubResourceData {
    pub unsafe fn create<T>(data: *const T, rowpitch: usize, slicepitch: usize) -> Self {
        let subresourcedata = win::D3D12_SUBRESOURCE_DATA {
            pData: data as *mut std::ffi::c_void,
            RowPitch: rowpitch as isize,
            SlicePitch: slicepitch as isize,
        };
        SSubResourceData {
            raw: subresourcedata,
        }
    }

    pub unsafe fn raw_mut(&mut self) -> &mut win::D3D12_SUBRESOURCE_DATA {
        &mut self.raw
    }
}

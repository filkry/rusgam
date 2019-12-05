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
        Self {
            resource: raw,
        }
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

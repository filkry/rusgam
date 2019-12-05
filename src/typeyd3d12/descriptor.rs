use super::*;

#[derive(Copy, Clone)]
pub enum EDescriptorHeapType {
    ConstantBufferShaderResourceUnorderedAccess,
    Sampler,
    RenderTarget,
    DepthStencil,
}

impl EDescriptorHeapType {
    pub fn d3dtype(&self) -> u32 {
        match self {
            EDescriptorHeapType::ConstantBufferShaderResourceUnorderedAccess => {
                D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV
            }
            EDescriptorHeapType::Sampler => D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
            EDescriptorHeapType::RenderTarget => D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            EDescriptorHeapType::DepthStencil => D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
        }
    }
}

#[derive(Clone)]
pub struct SDescriptorHeap {
    pub type_: EDescriptorHeapType,
    heap: ComPtr<ID3D12DescriptorHeap>,
}

impl SDescriptorHeap {
    pub unsafe fn new_from_raw(type_: EDescriptorHeapType, raw: ComPtr<ID3D12DescriptorHeap>) -> Self {
        Self {
            type_: type_,
            heap: raw,
        }
    }

    pub fn getcpudescriptorhandleforheapstart(&self) -> SDescriptorHandle {
        let start = unsafe { self.heap.GetCPUDescriptorHandleForHeapStart() };
        SDescriptorHandle { handle: start }
    }
}

pub struct SDescriptorHandle {
    handle: D3D12_CPU_DESCRIPTOR_HANDLE,
}

impl SDescriptorHandle {
    pub unsafe fn raw(&self) -> &D3D12_CPU_DESCRIPTOR_HANDLE {
        &self.handle
    }

    pub unsafe fn offset(&self, bytes: usize) -> SDescriptorHandle {
        SDescriptorHandle {
            handle: D3D12_CPU_DESCRIPTOR_HANDLE {
                ptr: self.handle.ptr + bytes,
            },
        }
    }
}

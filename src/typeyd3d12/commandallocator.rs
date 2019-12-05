use super::*;

#[derive(Clone)]
pub struct SCommandAllocator {
    type_: ECommandListType,
    commandallocator: ComPtr<ID3D12CommandAllocator>,
}

impl SCommandAllocator {
    pub unsafe fn new_from_raw(type_: ECommandListType, raw: ComPtr<ID3D12CommandAllocator>) -> Self {
        Self {
            type_: type_,
            commandallocator: raw,
        }
    }

    pub unsafe fn raw(&self) -> &ComPtr<ID3D12CommandAllocator> {
        &self.commandallocator
    }

    pub fn type_(&self) -> ECommandListType {
        self.type_
    }

    pub fn reset(&self) {
        unsafe { self.commandallocator.Reset() };
    }
}

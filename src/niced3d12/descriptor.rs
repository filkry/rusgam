use super::*;

pub struct SDescriptorHeap {
    pub(super) raw: t12::SDescriptorHeap,

    pub(super) numdescriptors: u32,
    pub(super) descriptorsize: usize,
    //cpudescriptorhandleforstart: t12::SDescriptorHandle<'heap, 'device>,
}

impl SDescriptorHeap {
    pub fn type_(&self) -> t12::EDescriptorHeapType {
        self.raw.type_
    }

    pub fn cpu_handle_heap_start(&self) -> t12::SCPUDescriptorHandle {
        self.raw.getcpudescriptorhandleforheapstart()
    }
}

impl SDescriptorHeap {
    pub fn cpu_handle(&self, index: usize) -> Result<t12::SCPUDescriptorHandle, &'static str> {
        if index < self.numdescriptors as usize {
            let offsetbytes: usize = (index * self.descriptorsize) as usize;
            let starthandle = self.cpu_handle_heap_start();
            Ok(unsafe { starthandle.offset(offsetbytes) })
        } else {
            Err("Descripter handle index past number of descriptors.")
        }
    }
}

use super::*;

pub enum EResourceMetadata {
    Invalid,
    SwapChainResource,
    BufferResource { count: usize, sizeofentry: usize },
    Texture2DResource,
}

pub struct SResource {
    pub(super) raw: t12::SResource,

    pub(super) metadata: EResourceMetadata,
}

impl SResource {
    pub fn create_vertex_buffer_view(&self) -> Result<t12::SVertexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource { count, sizeofentry } = self.metadata {
            Ok(t12::SVertexBufferView::create(
                self.raw.getgpuvirtualaddress(),
                (count * sizeofentry) as u32,
                sizeofentry as u32,
            ))
        } else {
            Err("Trying to create vertexbufferview for non-buffer resource")
        }
    }

    pub fn create_index_buffer_view(
        &self,
        format: t12::EDXGIFormat,
    ) -> Result<t12::SIndexBufferView, &'static str> {
        if let EResourceMetadata::BufferResource { count, sizeofentry } = self.metadata {
            Ok(t12::SIndexBufferView::create(
                self.raw.getgpuvirtualaddress(),
                format,
                (count * sizeofentry) as u32,
            ))
        } else {
            Err("Trying to create indexbufferview for non-buffer resource")
        }
    }
}

pub(super) fn update_subresources_stack(
    commandlist: &mut SCommandList,
    destinationresource: &mut SResource,
    intermediateresource: &mut SResource,
    intermediateoffset: u64,
    firstsubresource: u32,
    numsubresources: u32,
    srcdata: &mut t12::SSubResourceData,
) {
    unsafe {
        directxgraphicssamples::UpdateSubresourcesStack(
            commandlist.raw().raw().as_raw(),
            destinationresource.raw.raw_mut().as_raw(),
            intermediateresource.raw.raw_mut().as_raw(),
            intermediateoffset,
            firstsubresource,
            numsubresources,
            srcdata.raw_mut(),
        );
    }
}

impl Default for EResourceMetadata {
    fn default() -> Self {
        EResourceMetadata::Invalid
    }
}

impl t12::SSubResourceData {
    pub fn createbuffer<T>(data: &[T]) -> Self {
        let buffersize = data.len() * std::mem::size_of::<T>();
        unsafe { Self::create(data.as_ptr(), buffersize, buffersize) }
    }
}

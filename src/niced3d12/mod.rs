#![allow(dead_code)]

mod adapter;
mod commandallocator;
mod commandlist;
mod commandlistpool;
mod commandqueue;
mod descriptor;
pub mod descriptorallocator;
mod device;
mod dynamicdescriptorheap;
mod factory;
mod fence;
mod linearuploadbuffer;
mod pipelinestate;
mod resource;
mod rootsignature;
mod swapchain;
mod window;

use collections::{SPool, SPoolHandle};
use directxgraphicssamples;
use safewindows;
use typeyd3d12 as t12;

use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr;

// -- $$$FRK(TODO): all these imports should not exist
use winapi::shared::minwindef::*;
use winapi::um::d3d12sdklayers::*;

pub use self::adapter::*;
pub use self::commandallocator::*;
pub use self::commandlist::*;
pub use self::commandlistpool::*;
pub use self::commandqueue::*;
pub use self::descriptor::*;
pub use self::device::*;
pub use self::factory::*;
pub use self::fence::*;
pub use self::pipelinestate::*;
pub use self::resource::*;
pub use self::rootsignature::*;
pub use self::swapchain::*;
pub use self::window::SD3D12Window;

pub fn load_texture(device: &SDevice, cl: &mut SCommandList, file_path: &str) -> (SResource, SResource) {
    // $$$FRK(TODO): allocates
    let bytes = std::fs::read(file_path).unwrap();
    let tga = tinytga::Tga::from_slice(bytes.as_slice()).unwrap();

    // -- $$$FRK(TODO): allocates
    let mut pixels = Vec::new();

    for mut pixel in tga.into_iter() {
        pixel = pixel | (0xff << 24); // $$$FRK(HACK): max alpha
        pixels.push(pixel);
    }

    let mut resource = device
        .create_committed_texture2d_resource(
            t12::EHeapType::Default,
            tga.width() as u32,
            tga.height() as u32,
            1, // array size
            1, // mip levels
            t12::EDXGIFormat::R8G8B8A8UNorm,
            None,
            t12::SResourceFlags::none(),
            t12::EResourceStates::Common,
        )
        .unwrap();

    cl.transition_resource(
        &resource,
        t12::EResourceStates::Common,
        t12::EResourceStates::CopyDest,
    )
    .unwrap();

    let requiredsize = resource.get_required_intermediate_size(); // almost certainly wrong! look into d3d12.h GetIntermediateSize

    let mut intermediate_resource = device
        .create_committed_buffer_resource(
            t12::EHeapType::Upload,
            t12::EHeapFlags::ENone,
            t12::SResourceFlags::none(),
            t12::EResourceStates::GenericRead,
            1,
            requiredsize,
        )
        .unwrap();

    let mut data = t12::SSubResourceData::create_texture_2d(
        pixels.as_slice(),
        tga.width() as usize,
        tga.height() as usize,
    );

    update_subresources_stack(
        cl,
        &mut resource,
        &mut intermediate_resource,
        0,
        0,
        1,
        &mut data,
    );

    (intermediate_resource, resource)
}

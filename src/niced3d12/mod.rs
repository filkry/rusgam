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
use std::rc::{Rc};

// -- $$$FRK(TODO): all these imports should not exist
use winapi::shared::minwindef::*;
use winapi::um::d3d12sdklayers::*;

pub use self::adapter::*;
pub use self::commandallocator::*;
pub use self::commandlist::*;
pub use self::commandlistpool::*;
pub use self::commandqueue::*;
pub use self::descriptor::*;
pub use self::descriptorallocator::*;
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

pub fn create_committed_depth_textures<'a> (
    width: u32,
    height: u32,
    count: u16,
    device: &SDevice,
    format: t12::EDXGIFormat,
    initial_state: t12::EResourceStates,
    direct_command_pool: &mut SCommandListPool,
    depth_descriptor_allocator: &Rc<descriptorallocator::SDescriptorAllocator>,
) -> Result<(SResource, descriptorallocator::SDescriptorAllocatorAllocation), &'static str> {

    if depth_descriptor_allocator.type_() != t12::EDescriptorHeapType::DepthStencil {
        break_err!(Err("Non-DepthStencil descriptor allocator passed to create_depth_texture."));
    }

    direct_command_pool.flush_blocking().unwrap();

    let clear_value = t12::SClearValue {
        format: t12::EDXGIFormat::D32Float,
        value: t12::EClearValue::DepthStencil(t12::SDepthStencilValue {
            depth: 1.0,
            stencil: 0,
        }),
    };

    // -- need to not let this be destroyed
    let mut depth_texture_resource = device.create_committed_texture2d_resource(
        t12::EHeapType::Default,
        width,
        height,
        count as u16,
        0,
        format,
        //t12::EDXGIFormat::D32Float,
        Some(clear_value),
        t12::SResourceFlags::from(t12::EResourceFlags::AllowDepthStencil),
        initial_state,
    )?;

    let descriptors = descriptor_alloc(depth_descriptor_allocator, count as usize)?;

    if count == 1 {
        let desc = t12::SDepthStencilViewDesc {
            format: t12::EDXGIFormat::D32Float,
            view_dimension: t12::EDSVDimension::Texture2D,
            flags: t12::SDSVFlags::from(t12::EDSVFlags::None),
            data: t12::EDepthStencilViewDescData::Tex2D(t12::STex2DDSV { mip_slice: 0 }),
        };

        device.create_depth_stencil_view(
            &mut depth_texture_resource,
            &desc,
            descriptors.cpu_descriptor(0),
        )?;
    }
    else {
        for i in 0..count {
            let desc = t12::SDepthStencilViewDesc {
                format: t12::EDXGIFormat::D32Float,
                view_dimension: t12::EDSVDimension::Texture2DArray,
                flags: t12::SDSVFlags::from(t12::EDSVFlags::None),
                data: t12::EDepthStencilViewDescData::Tex2DArray(t12::STex2DArrayDSV{
                    mip_slice: 0,
                    first_array_slice: i as u32,
                    array_size: 1,
                }),
            };

            device.create_depth_stencil_view(
                &mut depth_texture_resource,
                &desc,
                descriptors.cpu_descriptor(i as usize),
            )?;
        }
    }

    Ok((depth_texture_resource, descriptors))
}

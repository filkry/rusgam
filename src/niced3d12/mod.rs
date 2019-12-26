#![allow(dead_code)]

mod adapter;
mod commandallocator;
mod commandlist;
mod commandlistpool;
mod commandqueue;
mod descriptor;
mod descriptorallocator;
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
use std::fs::File;

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

pub fn load_texture(cl: &mut SCommandList, file_path: &'static str) {
    let f = File::open(file_path);
}
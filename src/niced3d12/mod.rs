#![allow(dead_code)]

mod window;
mod factory;
mod adapter;
mod device;
mod swapchain;
mod commandlist;
mod commandallocator;
mod commandqueue;
mod fence;
mod resource;
mod descriptor;
mod commandlistpool;
mod pipelinestate;

use collections::{SPool, SPoolHandle};
use directxgraphicssamples;
use safewindows;
use typeyd3d12 as t12;

use std::cell::{RefCell};
use std::ops::{Deref};
use std::ptr;
use std::marker::{PhantomData};

// -- $$$FRK(TODO): all these imports should not exist
use winapi::shared::minwindef::*;
use winapi::um::d3d12sdklayers::*;

pub use self::factory::*;
pub use self::adapter::*;
pub use self::device::*;
pub use self::swapchain::*;
pub use self::commandlist::*;
pub use self::commandallocator::*;
pub use self::commandqueue::*;
pub use self::commandlistpool::*;
pub use self::fence::*;
pub use self::resource::*;
pub use self::descriptor::*;
pub use self::pipelinestate::*;
pub use self::window::SD3D12Window;

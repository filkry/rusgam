// -- $$$FRK(LICENSE): This file is under the MIT License per Microsoft
/*
The MIT License (MIT)

Copyright (c) 2015 Microsoft

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{mem, ptr};

use winapi::ctypes::c_void;
use winapi::shared::basetsd::{SIZE_T, UINT64};
use winapi::shared::dxgi::*;
use winapi::shared::dxgi1_2::*;
use winapi::shared::dxgi1_3::*;
use winapi::shared::dxgi1_4::*;
use winapi::shared::dxgi1_5::*;
use winapi::shared::dxgi1_6::*;
use winapi::shared::minwindef::*;
use winapi::shared::{dxgiformat, dxgitype, winerror};
use winapi::um::d3d12::*;
use winapi::um::d3d12sdklayers::*;
use winapi::um::{d3dcommon, unknwnbase};
use winapi::Interface;

pub unsafe fn MemcpySubresource(
    dest: *const D3D12_MEMCPY_DEST,
    src: *const D3D12_SUBRESOURCE_DATA,
    rowsizesinbytes: SIZE_T,
    numrows: UINT,
    numslices: UINT,
) {
    for z in 0isize..numslices as isize {
        let destoffset: isize = (*dest).SlicePitch as isize * z;
        let destslice: *mut BYTE = ((*dest).pData as *mut BYTE).offset(destoffset);
        let srcoffset: isize = (*src).SlicePitch as isize * z;
        let srcslice: *const BYTE = ((*src).pData as *const BYTE).offset(srcoffset);

        for y in 0isize..numrows as isize {
            ptr::copy_nonoverlapping(
                srcslice.offset((*src).RowPitch as isize * y),
                destslice.offset((*dest).RowPitch as isize * y),
                rowsizesinbytes,
            );
        }
    }
}

//------------------------------------------------------------------------------------------------
// All arrays must be populated (e.g. by calling GetCopyableFootprints)
pub unsafe fn UpdateSubresources(
    cmdlist: *mut ID3D12GraphicsCommandList,
    destinationresource: *mut ID3D12Resource,
    intermediate: *mut ID3D12Resource,
    firstsubresource: UINT,
    numsubresources: UINT,
    requiredsize: UINT64,
    layouts: *const D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    numrows: *const UINT,
    rowsizesinbytes: *const UINT64,
    srcdata: *const D3D12_SUBRESOURCE_DATA,
) -> UINT64 {
    assert!(firstsubresource <= D3D12_REQ_SUBRESOURCES);
    assert!(numsubresources <= D3D12_REQ_SUBRESOURCES - firstsubresource);

    // Minor validation
    let intermediatedesc = (*intermediate).GetDesc();
    let destinationdesc = (*destinationresource).GetDesc();
    let cond1 = intermediatedesc.Dimension != D3D12_RESOURCE_DIMENSION_BUFFER;
    let cond2 = intermediatedesc.Width < (requiredsize + (*layouts.offset(0)).Offset);
    let cond3 = destinationdesc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER
        && (firstsubresource != 0 || numsubresources != 1);
    if cond1 || cond2 || cond3 {
        panic!("No Err here yet");
    }

    let mut data: *mut BYTE = ptr::null_mut();
    let hr = (*intermediate).Map(0, ptr::null(), &mut data as *mut *mut _ as *mut *mut c_void);
    if winerror::FAILED(hr) {
        panic!("No Err here yet");
    }

    for i in 0..numsubresources {
        //if (*rowsizesinbytes.offset(i)) > SIZE_T(-1)) return 0;
        let layout = layouts.offset(i as isize);
        let dataoffset: isize = (*layout).Offset as isize;
        let destdata = D3D12_MEMCPY_DEST {
            pData: data.offset(dataoffset) as *mut c_void,
            RowPitch: (*layout).Footprint.RowPitch as usize,
            SlicePitch: (*layout).Footprint.RowPitch as SIZE_T
                * *(numrows.offset(i as isize)) as SIZE_T,
        };
        MemcpySubresource(
            &destdata,
            srcdata.offset(i as isize),
            *(rowsizesinbytes.offset(i as isize)) as SIZE_T,
            *(numrows.offset(i as isize)),
            (*layout).Footprint.Depth,
        );
    }
    (*intermediate).Unmap(0, ptr::null());

    if destinationdesc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER {
        (*cmdlist).CopyBufferRegion(
            destinationresource,
            0,
            intermediate,
            (*layouts).Offset,
            (*layouts).Footprint.Width as u64,
        );
    } else {
        for i in 0..numsubresources {
            let layout = layouts.offset(i as isize);
            let mut dst = CD3DX12_TEXTURE_COPY_LOCATION::from_res_sub(
                destinationresource,
                i + firstsubresource,
            );
            let mut src = CD3DX12_TEXTURE_COPY_LOCATION::from_res_footprint(intermediate, *layout);
            (*cmdlist).CopyTextureRegion(&mut dst, 0, 0, 0, &mut src, ptr::null());
        }
    }
    return requiredsize;
}

// Stack-allocating UpdateSubresources implementation
pub unsafe fn UpdateSubresourcesStack(
    cmdlist: *mut ID3D12GraphicsCommandList,
    destinationresource: *mut ID3D12Resource,
    intermediate: *mut ID3D12Resource,
    intermediateoffset: UINT64,
    firstsubresource: UINT,
    numsubresources: UINT,
    srcdata: *mut D3D12_SUBRESOURCE_DATA,
) -> UINT64 {
    assert!(numsubresources <= 10);

    let mut requiredsize: u64 = 0;
    let mut layouts: [D3D12_PLACED_SUBRESOURCE_FOOTPRINT; 10] = mem::zeroed();
    let mut numrows: [UINT; 10] = [0; 10];
    let mut rowsizesinbytes: [UINT64; 10] = [0; 10];

    let desc = (*destinationresource).GetDesc();
    let mut device: *mut ID3D12Device = ptr::null_mut();
    (*destinationresource).GetDevice(
        &ID3D12Device::uuidof(),
        &mut device as *mut *mut _ as *mut *mut c_void,
    );
    (*device).GetCopyableFootprints(
        &desc,
        firstsubresource,
        numsubresources,
        intermediateoffset,
        layouts.as_mut_ptr(),
        numrows.as_mut_ptr(),
        rowsizesinbytes.as_mut_ptr(),
        &mut requiredsize,
    );
    (*device).Release();

    return UpdateSubresources(
        cmdlist,
        destinationresource,
        intermediate,
        firstsubresource,
        numsubresources,
        requiredsize,
        layouts.as_mut_ptr(),
        numrows.as_mut_ptr(),
        rowsizesinbytes.as_mut_ptr(),
        srcdata,
    );
}

pub struct CD3DX12_TEXTURE_COPY_LOCATION {}

impl CD3DX12_TEXTURE_COPY_LOCATION {
    pub unsafe fn from_res(res: *mut ID3D12Resource) -> D3D12_TEXTURE_COPY_LOCATION {
        D3D12_TEXTURE_COPY_LOCATION {
            pResource: res,
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            u: mem::zeroed(),
        }
    }

    pub unsafe fn from_res_footprint(
        res: *mut ID3D12Resource,
        footprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    ) -> D3D12_TEXTURE_COPY_LOCATION {
        let mut result: D3D12_TEXTURE_COPY_LOCATION = mem::zeroed();
        result.pResource = res;
        result.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
        *(result.u.PlacedFootprint_mut()) = footprint;
        result
    }

    pub unsafe fn from_res_sub(res: *mut ID3D12Resource, sub: UINT) -> D3D12_TEXTURE_COPY_LOCATION {
        let mut result: D3D12_TEXTURE_COPY_LOCATION = mem::zeroed();
        result.pResource = res;
        result.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        *(result.u.SubresourceIndex_mut()) = sub;
        result
    }
}


pub unsafe fn get_required_intermediate_size(
    destination_resource: *mut ID3D12Resource,
    first_subresource: UINT,
    num_subresources: UINT,
) -> u64 {
    let desc = destination_resource.as_ref().unwrap().GetDesc();
    let mut required_size = 0;

    let mut device : *mut ID3D12Device = std::ptr::null_mut();
    destination_resource.as_ref().unwrap().GetDevice(&ID3D12Device::uuidof(), &mut device as *mut *mut _ as *mut *mut c_void);
    device.as_ref().unwrap().GetCopyableFootprints(
        &desc,
        first_subresource,
        num_subresources,
        0,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut required_size,
    );
    device.as_ref().unwrap().Release();

    return required_size;
}
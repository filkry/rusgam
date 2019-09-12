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

use std::{mem, ptr};

use winapi::ctypes::c_void;
use winapi::shared::dxgi::*;
use winapi::shared::dxgi1_2::*;
use winapi::shared::dxgi1_3::*;
use winapi::shared::dxgi1_4::*;
use winapi::shared::dxgi1_5::*;
use winapi::shared::dxgi1_6::*;
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::*;
use winapi::shared::{dxgiformat, dxgitype, winerror};
use winapi::um::d3d12::*;
use winapi::um::d3d12sdklayers::*;
use winapi::um::{d3dcommon, unknwnbase};
use winapi::Interface;

unsafe fn MemcpySubresource(
    dest: *const D3D12_MEMCPY_DEST,
    src: *const D3D12_SUBRESOURCE_DATA,
    rowsizesinbytes: SIZE_T,
    numrows: UINT,
    numslices: UINT,
)
{
    for z in 0..numslices {
        let destoffset : isize = (*dest).SlicePitch * z;
        let destslice : *const BYTE = ((*dest).pData as *const BYTE).offset(destoffset);
        let srcoffset : isize = (*src).SlicePitch * z;
        let srcslice : *const BYTE = ((*src).pData as *const BYTE).offset(srcoffset);

        for y in 0..numrows {
            ptr::copy_nonoverlapping(
                srcslice.offset((*src).RowPitch * y),
                destslice.offset((*dest).RowPitch * y),
                rowsizesinbytes);
        }
    }
}

/*
// -- $$$FRK(TODO) go back to raw D3D types, more literal port
unsafe fn UpdateSubresources(
    cmdlist: &SCommandList,
    destinationresource: &SResource,
    intermediate: &SResource,
    usize: firstsubresource,
    usize: numsubresources,
    requiredsize: u64,
    layouts: &[D3D12_PLACED_SUBRESOURCE_FOOTPRINT],
    numrows: &[usize],
    rowsizeinbytes: &[u64],
    srcdata: &[D3D12_SUBRESOURCE_DATA],
) -> u64
{
    assert!(firstsubresource < D3D12_REQ_SUBRESOURCES);
    assert!(numsubresources < D3D12_REQ_SUBRESOURCES - firstsubresource);
    assert_eq!(numsubresources, playouts.len());
    assert_eq!(numsubresources, numrows.len());
    assert_eq!(numsubresources, rowsizesinbytes.len());
    assert_eq!(numsubresources, srcdata.len());

    // Minor validation
    let intermediatedesc = intermediate.getdesc();
    let destinationdesc = destinationresource.getdesc();

    if (intermediatedesc.Dimension != D3D12_RESOURCE_DIMENSION_BUFFER ||
        intermediatedesc.Width < requiredsize + layouts[0].Offset ||
        (destinationdesc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER &&
            (firstsubresource != 0 || numsubresources != 1)))
    {
        return 0;
    }

    let data : *mut u8 = ptr::null_mut();
    let hr = intermediate.map(0, ptr::null(), *mut data as *mut *mut c_void);
    returnerrifwinerror!(hr, "Failed to map to intermediateresource");

    for i in 0..numsubresources {
        if rowsizesinbytes[i] > std::u64::MAX { return 0; }

        let destdata = D3D12_MEMCPY_DEST{
            pData: data.offset(layouts[i].Offset),
            RowPitch: layouts[i].Footprint.RowPitch,
            SlicePitch: layouts[i].Footprint.RowPitch * numrows[i],
        };
        MemcpySubresource(&DestData, &pSrcData[i], static_cast<SIZE_T>(pRowSizesInBytes[i]), pNumRows[i], pLayouts[i].Footprint.Depth);
    }
    pIntermediate->Unmap(0, nullptr);

    if (DestinationDesc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER)
    {
        pCmdList->CopyBufferRegion(
            pDestinationResource, 0, pIntermediate, pLayouts[0].Offset, pLayouts[0].Footprint.Width);
    }
    else
    {
        for (UINT i = 0; i < NumSubresources; ++i)
        {
            CD3DX12_TEXTURE_COPY_LOCATION Dst(pDestinationResource, i + FirstSubresource);
            CD3DX12_TEXTURE_COPY_LOCATION Src(pIntermediate, pLayouts[i]);
            pCmdList->CopyTextureRegion(&Dst, 0, 0, 0, &Src, nullptr);
        }
    }
    return RequiredSize;
}
*/

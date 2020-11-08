use glm::{Vec3, Vec2};
use typeyd3d12 as t12;

// -- must match SBaseVertexData in vertex.hlsl
#[repr(C)]
pub struct SBaseVertexData {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

impl SBaseVertexData {
    pub fn new_input_elements(slot: usize) -> [t12::SInputElementDesc; 3] {
        [
            t12::SInputElementDesc::create(
                "POSITION",
                0,
                t12::EDXGIFormat::R32G32B32Float,
                slot as u32,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
            t12::SInputElementDesc::create(
                "NORMAL",
                0,
                t12::EDXGIFormat::R32G32B32Float,
                slot as u32,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
            t12::SInputElementDesc::create(
                "TEXCOORD",
                0,
                t12::EDXGIFormat::R32G32Float,
                slot as u32,
                winapi::um::d3d12::D3D12_APPEND_ALIGNED_ELEMENT,
                t12::EInputClassification::PerVertexData,
                0,
            ),
        ]
    }
}

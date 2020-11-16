struct SPixelShaderInput {
    float4 position : SV_Position;
    float3 world_position: POSITION2;
    float4 normal   : NORMAL;
    float2 uv       : TEXCOORD;
};

// -- must match SModelViewProjection in render/shaderbindings/types.rs
struct SModelViewProjection
{
    matrix model;
    matrix viewprojection;
    matrix mvp;
};
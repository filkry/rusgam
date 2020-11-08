struct SPixelShaderInput {
    float4 position : SV_Position;
    float3 world_position: POSITION2;
    float4 normal   : NORMAL;
    float2 uv       : TEXCOORD;
};

// -- must match SBaseVertexData in render/shaderbindings/types.rs
struct SBaseVertexData
{
    float3 position : POSITION;
    float3 normal   : NORMAL;
    float2 uv       : TEXCOORD;
};

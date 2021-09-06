static const float PI = 3.14159265f;

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

// -- must match SIntanceData in vertex_hlsl_bind.rs
struct SInstanceData {
    matrix model_location;
    u32 texture_metadata_index;
};

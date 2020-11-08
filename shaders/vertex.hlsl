#include "types.hlsl"

struct SModelViewProjection
{
    matrix model;
    matrix viewprojection;
    matrix mvp;
};

ConstantBuffer<SModelViewProjection> modelviewprojectionconstantbuffer : register(b0);

SPixelShaderInput main(SBaseVertexData input)
{
    SPixelShaderInput output;

    output.position = mul(modelviewprojectionconstantbuffer.mvp, float4(input.position, 1.0f));
    output.world_position = mul(modelviewprojectionconstantbuffer.model, float4(input.position, 1.0)).xyz;
    output.normal = mul(modelviewprojectionconstantbuffer.model, float4(input.normal, 0.0f));
    output.uv = input.uv;

    return output;
}
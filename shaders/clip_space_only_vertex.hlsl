#include "types.hlsl"

ConstantBuffer<SModelViewProjection> modelviewprojectionconstantbuffer : register(b0);

SPixelShaderInput main(float3 local_vertex: POSITION)
{
    SPixelShaderInput output;

    output.position = mul(modelviewprojectionconstantbuffer.mvp, float4(local_vertex, 1.0f));

    return output;
}
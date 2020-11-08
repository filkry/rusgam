#include "types.hlsl"

ConstantBuffer<SModelViewProjection> modelviewprojectionconstantbuffer : register(b0);

SPixelShaderInput main(SBaseVertexData input)
{
    SPixelShaderInput output;

    output.position = mul(modelviewprojectionconstantbuffer.mvp, float4(input.position, 1.0f));

    return output;
}
//#include "types.hlsl"

ConstantBuffer<SModelViewProjection> modelviewprojectionconstantbuffer : register(b0);

SPixelShaderInput main(float3 local_vertex: POSITION, float3 local_normal: NORMAL, float2 uv: TEXCOORD)
{
    SPixelShaderInput output;

    output.position = mul(modelviewprojectionconstantbuffer.mvp, float4(local_vertex, 1.0));
    output.world_position = mul(modelviewprojectionconstantbuffer.model, float4(local_vertex, 1.0)).xyz;
    output.normal = mul(modelviewprojectionconstantbuffer.model, float4(local_normal, 0.0));
    output.uv = uv;

    return output;
}
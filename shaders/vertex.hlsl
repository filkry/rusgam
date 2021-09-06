#include "types.hlsl"

ConstantBuffer<matrix> view_projection : register(b0);

StructuredBuffer<SInstanceData> instance_metadata_buffer : register(t0, space0);
StructuredBuffer<float3> vertex_buffer : register(t1, space0);
StructuredBuffer<float3> normal_buffer : register(t2, space0);
StructuredBuffer<float2> uv_buffer : register(t3, space0);

SPixelShaderInput main()
{
    SPixelShaderInput output;

    // -- $$$FRK(TODO): precompute all these per-instance in compute shader
    matrix model = instance_metadata_buffer[SV_InstanceID].model_location;
    matrix mvp = mul(model, view_projection);

    float3 local_vertex = vertex_buffer[SV_VertexID];
    float3 local_normal = normal_buffer[SV_VertexID];
    float2 uv = uv_buffer[SV_VertexID];

    output.position = mul(mvp, float4(local_vertex, 1.0));
    output.world_position = mul(model, float4(local_vertex, 1.0)).xyz;
    output.normal = mul(model, float4(local_normal, 0.0));
    output.uv = uv;

    return output;
}
// -- must match SVertexPosColourUV/model_per_vertex_input_layout_desc in model.rs
struct SVertexPosColorUV
{
    float3 position : POSITION;
    float3 normal   : NORMAL;
    float2 uv       : TEXCOORD;
    uint joints[4] : JOINTS;
    float4 joint_weights: JOINTWEIGHTS;
};

struct SModelViewProjection
{
    matrix model;
    matrix viewprojection;
    matrix mvp;
};

ConstantBuffer<SModelViewProjection> modelviewprojectionconstantbuffer : register(b0);
StructuredBuffer<matrix> jointworldtransforms : register(t0);

struct SVertexShaderOutput
{
    float4 position : SV_Position;
    float3 world_position: POSITION2;
    float4 normal   : NORMAL;
    float2 uv       : TEXCOORD;
};

SVertexShaderOutput main(SVertexPosColorUV input)
{
    SVertexShaderOutput output;

    matrix vertmat = mul(input.joint_weights[0], jointworldtransforms[input.joints[0]]) +
                     mul(input.joint_weights[1], jointworldtransforms[input.joints[1]]) +
                     mul(input.joint_weights[2], jointworldtransforms[input.joints[2]]) +
                     mul(input.joint_weights[3], jointworldtransforms[input.joints[3]]);
    float4 world_pos = mul(vertmat, float4(input.position, 1.0));
    float4 world_normal = mul(vertmat, float4(input.normal, 1.0));
    output.position = mul(modelviewprojectionconstantbuffer.viewprojection, world_pos);
    output.world_position = world_pos.xyz;
    output.normal = mul(modelviewprojectionconstantbuffer.viewprojection, world_normal);
    output.uv = input.uv;

    return output;
}
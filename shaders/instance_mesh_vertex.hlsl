// -- must match SVertexPosColourUV in model.rs + SDebugSphereShaderInstance in temp.rs
struct SVertexPosColorUV
{
    float3 position : POSITION;
    float3 normal   : NORMAL;
    float2 uv       : TEXCOORD;
    float instance_scale: INSTANCESCALE;
    float3 instance_position : INSTANCEPOSITION;
    float4 colour : COLOR;
};

struct SViewProjection
{
    matrix vp;
};

ConstantBuffer<SViewProjection> viewprojectionconstantbuffer : register(b0);

struct SVertexShaderOutput
{
    float4 position : SV_Position;
    float2 uv       : TEXCOORD;
    float4 colour       : COLOR;
};

SVertexShaderOutput main(SVertexPosColorUV input)
{
    SVertexShaderOutput output;

    output.position = mul(viewprojectionconstantbuffer.vp, float4(input.position * input.instance_scale + input.instance_position, 1.0f));
    output.uv = input.uv;
    output.colour = input.colour;

    return output;
}
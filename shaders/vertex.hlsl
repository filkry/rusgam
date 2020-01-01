struct SVertexPosColorUV
{
    float3 position : POSITION;
    float3 color    : COLOR;
    float3 normal   : NORMAL;
    float2 uv       : TEXCOORD;
};

struct SModelViewProjection
{
    matrix model;
    matrix viewprojection;
    matrix mvp;
};

ConstantBuffer<SModelViewProjection> modelviewprojectionconstantbuffer : register(b0);

struct SVertexShaderOutput
{
    float4 color    : COLOR;
    float4 position : SV_Position;
    float3 world_position: POSITION2;
    float4 normal   : NORMAL;
    float2 uv       : TEXCOORD;
};

SVertexShaderOutput main(SVertexPosColorUV input)
{
    SVertexShaderOutput output;

    output.position = mul(modelviewprojectionconstantbuffer.mvp, float4(input.position, 1.0f));
    output.world_position = mul(modelviewprojectionconstantbuffer.model, float4(input.position, 1.0)).xyz;
    output.color = float4(input.color, 1.0f);
    output.normal = mul(modelviewprojectionconstantbuffer.model, float4(input.normal, 0.0f));
    output.uv = input.uv;

    return output;
}
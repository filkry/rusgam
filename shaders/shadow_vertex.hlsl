struct SVertexPosColorUV
{
    float3 position : POSITION;
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
    float4 position : SV_Position;
};

SVertexShaderOutput main(SVertexPosColorUV input)
{
    SVertexShaderOutput output;

    output.position = mul(modelviewprojectionconstantbuffer.mvp, float4(input.position, 1.0f));

    return output;
}
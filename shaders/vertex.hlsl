struct SVertexPosColorUV
{
    float3 position : POSITION;
    float3 color    : COLOR;
    float2 uv       : TEXCOORD;
};

struct SModelViewProjection
{
    matrix mvp;
};

ConstantBuffer<SModelViewProjection> modelviewprojectionconstantbuffer : register(b0);

struct SVertexShaderOutput
{
    float4 color    : COLOR;
    float4 position : SV_Position;
    float2 uv       : TEXCOORD;
};

SVertexShaderOutput main(SVertexPosColorUV input)
{
    SVertexShaderOutput output;

    output.position = mul(modelviewprojectionconstantbuffer.mvp, float4(input.position, 1.0f));
    output.color = float4(input.color, 1.0f);
    output.uv = input.uv;

    return output;
}
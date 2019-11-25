struct SVertexPosColor
{
    float3 position : POSITION;
    float3 color    : COLOR;
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
};

SVertexShaderOutput main(SVertexPosColor input)
{
    SVertexShaderOutput output;

    output.position = mul(modelviewprojectionconstantbuffer.mvp, float4(input.position, 1.0f));
    output.color = float4(input.color, 1.0f);

    return output;
}
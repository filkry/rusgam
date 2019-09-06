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

SVertexShaderOutput main(SVertexPosColor in)
{
    SVertexShaderOutput out;

    out.position = mul(modelviewprojectionconstantbuffer.mvp, float4(in.position, 1.0f));
    out.color = float4(in.color, 1.0f);

    return out;
}
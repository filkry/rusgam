// -- must match SDebugLineShaderVert and corresponding layout desc in render.rs
struct SDebugLineShaderVert
{
    float3 position : POSITION;
    float3 color   : COLOR;
};

struct SViewProjection {
    matrix vp;
};

ConstantBuffer<SViewProjection> viewprojection : register(b0);

struct SVertexShaderOutput
{
    float4 position : SV_Position;
    float3 color : COLOR;
};

SVertexShaderOutput main(SDebugLineShaderVert input)
{
    SVertexShaderOutput output;

    output.position = mul(viewprojection.vp, float4(input.position, 1.0));
    output.color = input.color;

    return output;
}
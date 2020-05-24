// -- must match SDebugPointShaderVert and corresponding layout desc in render.rs
struct SDebugPointShaderVert
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
    float4 color : COLOR;
};

SVertexShaderOutput main(SDebugPointShaderVert input)
{
    SVertexShaderOutput output;

    output.position = mul(viewprojection.vp, float4(input.position, 1.0));
    output.color = float4(input.color, 1.0);

    return output;
}
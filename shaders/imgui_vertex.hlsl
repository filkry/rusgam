// -- must match imgui::DrawVert
struct SDrawVert
{
    float2 pos : POSITION;
    float2 uv  : TEXCOORD;
    uint color : BLENDINDICES; // packed 4 x 1byte array
};

struct SOrthoMat
{
    matrix orthomat;
};

ConstantBuffer<SOrthoMat> orthomatbuffer : register(b0);

struct SVertexShaderOutput
{
    float4 position : SV_POSITION;
    float2 uv  : TEXCOORD;
};

SVertexShaderOutput main(SDrawVert input)
{
    SVertexShaderOutput output;

    output.position = mul(orthomatbuffer.orthomat, float4(input.pos.xy, 0.0, 1.0));
    output.uv = input.uv;

    return output;
}
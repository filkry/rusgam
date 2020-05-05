// -- must match imgui::DrawVert
struct SDrawVert
{
    float2 pos : POSITION;
    float2 uv  : TEXCOORD;
    uint color : COLOR; // packed 4 x 1byte array
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
    float4 colour: COLOR;
};

SVertexShaderOutput main(SDrawVert input)
{
    SVertexShaderOutput output;

    float4 colour;
    colour.x = (float)(input.color & 0x000000FF) / 255.0;
    colour.y = (float)((input.color >> 8) & 0x000000FF) / 255.0;
    colour.z = (float)((input.color >> 16) & 0x000000FF) / 255.0;
    colour.w = (float)((input.color >> 24) & 0x000000FF) / 255.0;

    output.position = mul(orthomatbuffer.orthomat, float4(input.pos.xy, 0.0, 1.0));
    output.uv = input.uv;
    output.colour = colour;

    return output;
}
struct SPixelShaderInput
{
    float4 position : SV_POSITION;
    float2 uv  : TEXCOORD;
    float4 colour : COLOR;
};

Texture2D g_texture : register(t0);
SamplerState g_sampler : register(s0);

float4 main( SPixelShaderInput input ) : SV_Target
{
    float4 out_colour = input.colour;
    float4 tex_sample = g_texture.Sample(g_sampler, input.uv);
    out_colour.w = input.colour.w * tex_sample.w;

    return out_colour;
}
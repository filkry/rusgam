struct SPixelShaderInput
{
    float4 position : SV_POSITION;
    float2 uv  : TEXCOORD;
};

Texture2D g_texture : register(t0);
SamplerState g_sampler : register(s0);

float4 main( SPixelShaderInput input ) : SV_Target
{
    float4 out_colour = g_texture.Sample(g_sampler, input.uv);
    return out_colour;
}
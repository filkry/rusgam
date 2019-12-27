struct SPixelShaderInput
{
    float4 color    : COLOR;
    float4 position : SV_Position;
    float2 uv       : TEXCOORD;
};

Texture2D g_texture : register(t0);
SamplerState g_sampler : register(s0);

float4 main( SPixelShaderInput input ) : SV_Target
{
    return g_texture.Sample(g_sampler, input.uv);
    //return input.color;
}
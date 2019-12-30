struct SPixelShaderInput
{
    float4 color    : COLOR;
    float4 position : SV_Position;
    float2 uv       : TEXCOORD;
};

struct STextureMetadata {
    float is_textured;
};

ConstantBuffer<STextureMetadata> texture_metadata_buffer : register(b1);

Texture2D g_texture : register(t0);
SamplerState g_sampler : register(s0);

float4 main( SPixelShaderInput input ) : SV_Target
{
    if(texture_metadata_buffer.is_textured > 0.0f)
        return g_texture.Sample(g_sampler, input.uv);
    else
        return input.color;
}
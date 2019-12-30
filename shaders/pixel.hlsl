struct SPixelShaderInput
{
    float4 color    : COLOR;
    float4 position : SV_Position;
    float4 normal   : NORMAL;
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
    float3 light_dir = normalize(float3(-1.0, -1.0, -1.0));
    float simple_light_weight = saturate(dot(light_dir, -input.normal.xyz));

    float4 base_colour;
    if(texture_metadata_buffer.is_textured > 0.0f)
        base_colour = g_texture.Sample(g_sampler, input.uv);
    else
        base_colour = input.color;

    return base_colour * simple_light_weight;
}
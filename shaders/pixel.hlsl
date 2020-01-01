struct SPixelShaderInput
{
    float4 color    : COLOR;
    float4 position : SV_Position;
    float3 world_position: POSITION2;
    float4 normal   : NORMAL;
    float2 uv       : TEXCOORD;
};

struct STextureMetadata {
    float is_textured;
};

ConstantBuffer<STextureMetadata> texture_metadata_buffer : register(b1);

Texture2D g_texture : register(t0);
SamplerState g_sampler : register(s0);

TextureCube g_shadow_cube : register(t0, space1);
SamplerState g_shadow_sampler : register(s0, space1);

static const float PI = 3.14159265f;

float4 main( SPixelShaderInput input ) : SV_Target
{
    float3 light_pos = float3(0.0, 0.0, 0.0);
    float light_power = 50.0;
    //float3 light_dir = normalize(float3(-1.0, -1.0, -1.0));
    //float simple_light_weight = saturate(dot(light_dir, -input.normal.xyz));

    float3 to_light = light_pos - input.world_position;
    float dist_to_light = length(to_light);

    float3 to_light_dir = to_light / dist_to_light;

    float cos_theta = dot(to_light_dir, input.normal.xyz);

    float point_irradiance = (light_power * cos_theta) / (4.0 * PI * dist_to_light);

    float3 from_origin = input.world_position - float3(3.0, 0.0, 0.0);

    //float3 shadow_sample = g_shadow_cube.Sample(g_sampler, to_light_dir);
    //float4 shadow_sample = g_shadow_cube.Sample(g_shadow_sampler, from_origin);
    float4 shadow_sample = g_shadow_cube.Sample(g_shadow_sampler, from_origin);
    if(shadow_sample.x < 1.0) shadow_sample.x = 0.0;


    //float4 base_colour;
    //if(texture_metadata_buffer.is_textured > 0.0f)
    //    base_colour = g_texture.Sample(g_sampler, input.uv);
    //else
    //    base_colour = input.color;

    //return base_colour;
    //return base_colour * point_irradiance;
    //return (shadow_sample >= 1.0);
    return float4(shadow_sample);
}
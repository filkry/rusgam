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
    float always_diffuse_colour;
};

ConstantBuffer<STextureMetadata> texture_metadata_buffer : register(b1);

Texture2D g_texture : register(t0);
SamplerState g_sampler : register(s0);

TextureCube g_shadow_cube : register(t0, space1);
SamplerState g_shadow_sampler : register(s0, space1);

static const float PI = 3.14159265f;

float4 main( SPixelShaderInput input ) : SV_Target
{
    float3 light_pos = float3(5.0, 5.0, 5.0);
    float light_power = 50.0;
    //float3 light_dir = normalize(float3(-1.0, -1.0, -1.0));
    //float simple_light_weight = saturate(dot(light_dir, -input.normal.xyz));

    float3 to_light = light_pos - input.world_position;
    float dist_to_light = length(to_light);

    float3 to_light_dir = to_light / dist_to_light;
    float cos_theta = dot(to_light_dir, input.normal.xyz);
    float point_irradiance = (light_power * cos_theta) / (4.0 * PI * dist_to_light);

    float from_light_z = 0.0;
    if(abs(to_light.x) > abs(to_light.y) && abs(to_light.x) > abs(to_light.z)) {
        from_light_z = abs(to_light.x);
    }
    else if(abs(to_light.y) > abs(to_light.z)) {
        from_light_z = abs(to_light.y);
    }
    else {
        from_light_z = abs(to_light.z);
    }

    float3 shadow_sample_pos = -to_light + 0.1 * input.normal.xyz;

    float4 shadow_sample = g_shadow_cube.Sample(g_shadow_sampler, shadow_sample_pos);

    // -- from MJP's blog (https://mynameismjp.wordpress.com/2010/09/05/position-from-depth-3/)
    float far_clip_distance = 100.0;
    float near_clip_distance = 0.1;

    float projection_a = far_clip_distance / (far_clip_distance - near_clip_distance);
    float projection_b = -(far_clip_distance * near_clip_distance) / (far_clip_distance - near_clip_distance);
    float shadow_sample_light_space_z = projection_b / (shadow_sample.x - projection_a);

    float4 output = float4(0.0, 0.0, 0.0, 1.0);

    if(from_light_z >= (shadow_sample_light_space_z + 0.05)) {
        point_irradiance = 0.0; // obscured by shadow
    }

    float4 base_colour;
    if(texture_metadata_buffer.is_textured > 0.0f)
        base_colour = g_texture.Sample(g_sampler, input.uv);
    else
        base_colour = input.color;

    if(texture_metadata_buffer.always_diffuse_colour > 0.0)
        return base_colour;

    //return base_colour;
    return base_colour * point_irradiance;
    //return output;
}
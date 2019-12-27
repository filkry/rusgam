struct SPixelShaderInput
{
    float4 color    : COLOR;
    float4 position : SV_Position;
};

Texture2D g_texture : register(t0);

float4 main( SPixelShaderInput input ) : SV_Target
{
    return input.color;
}
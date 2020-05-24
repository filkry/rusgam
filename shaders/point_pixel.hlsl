struct SPixelShaderInput
{
    float4 position : SV_Position;
    float4 color : COLOR;
};

float4 main( SPixelShaderInput input ) : SV_Target
{
    return input.color;
}
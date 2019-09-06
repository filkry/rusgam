struct SPixelShaderInput
{
    float4 color    : COLOR;
    float4 position : SV_Position;
};

float4 main( SPixelShaderInput in ) : SV_Target
{
    return in.color;
}
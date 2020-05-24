struct SPixelShaderInput
{
    float4 position : SV_Position;
    float2 uv       : TEXCOORD;
    float4 colour       : COLOR;
};

float4 main( SPixelShaderInput input ) : SV_Target
{
    float4x4 alpha_dither_thresholds = {
        1.0 / 17.0,  9.0 / 17.0,  3.0 / 17.0, 11.0 / 17.0,
        13.0 / 17.0,  5.0 / 17.0, 15.0 / 17.0,  7.0 / 17.0,
        4.0 / 17.0, 12.0 / 17.0,  2.0 / 17.0, 10.0 / 17.0,
        16.0 / 17.0,  8.0 / 17.0, 14.0 / 17.0,  6.0 / 17.0
    };
    uint x_coord = floor(input.position.x);
    uint y_coord = floor(input.position.y);

    clip(input.colour.w < alpha_dither_thresholds[x_coord % 4][y_coord % 4] ? -1.0 : 1.0);

    return input.colour;
}
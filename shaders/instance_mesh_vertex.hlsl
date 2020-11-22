struct SVertInput {
    float3 position : POSITION;
    float3 normal   : NORMAL;
    float2 uv       : TEXCOORD;
};

struct SInstanceInput {
    float instance_scale: INSTANCESCALE;
    float3 instance_position : INSTANCEPOSITION;
    float4 colour : COLOR;
};

struct SViewProjection
{
    matrix vp;
};

ConstantBuffer<SViewProjection> viewprojectionconstantbuffer : register(b0);

struct SVertexShaderOutput
{
    float4 position : SV_Position;
    float2 uv       : TEXCOORD;
    float4 colour       : COLOR;
};

SVertexShaderOutput main(SVertInput vert, SInstanceInput instance)
{
    SVertexShaderOutput output;

    output.position = mul(viewprojectionconstantbuffer.vp, float4(vert.position * instance.instance_scale + instance.instance_position, 1.0f));
    output.uv = vert.uv;
    output.colour = instance.colour;

    return output;
}
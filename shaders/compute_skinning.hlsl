#include "types.hlsl"

struct SVertexSkinning {
    uint4 joints : JOINTS;
    float4 joint_weights: JOINTWEIGHTS;
};

// -- input
StructuredBuffer<matrix> joint_world_transforms : register(t0);
StructuredBuffer<float3> local_verts : register(t1);
StructuredBuffer<float3> local_normals : register(t2);
StructuredBuffer<SVertexSkinning> vertex_skinning : register(t3);

// -- output
RWStructuredBuffer<float3> skinned_verts : register(u0);
RWStructuredBuffer<float3> skinned_normals : register(u1);

[numthreads( 64, 1, 1 )]
void main(uint3 global_thread_id : SV_DispatchThreadID)
{
    uint vidx = global_thread_id.x;

    uint4 joints = vertex_skinning[vidx].joints;
    uint4 joint_weights = vertex_skinning[vidx].joint_weights;

    matrix vertmat = mul(joint_weights[0], joint_world_transforms[joints[0]]) +
                     mul(joint_weights[1], joint_world_transforms[joints[1]]) +
                     mul(joint_weights[2], joint_world_transforms[joints[2]]) +
                     mul(joint_weights[3], joint_world_transforms[joints[3]]);

    float3 world_vert_pos = mul(vertmat, float4(local_verts[vidx], 1.0));
    float3 world_normal = mul(vertmat, float4(local_normals[vidx], 0.0));

    skinned_verts[vidx] = world_vert_pos;
    skinned_normals[vidx] = world_normal;
}
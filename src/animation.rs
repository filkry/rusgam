use allocate::{TMemAllocator, SMemVec};
use glm::{Vec3, Quat};
use model::{SMeshSkinning};
use utils::{STransform, lerp, unlerp_f32, gltf_accessor_slice, clamp};

pub struct SAnimation<'a> {
    pub duration: f32,
    channels: SMemVec<'a, EChannel<'a>>,
    bound_node_to_joint_map: SMemVec<'a, Option<usize>>,
}

struct STranslationChannel<'a> {
    node: usize,
    sample_times: SMemVec<'a, f32>,
    sample_values: SMemVec<'a, Vec3>,
}

struct SRotationChannel<'a> {
    node: usize,
    sample_times: SMemVec<'a, f32>,
    sample_values: SMemVec<'a, Quat>,
}

struct SScaleChannel<'a> {
    node: usize,
    sample_times: SMemVec<'a, f32>,
    sample_values: SMemVec<'a, f32>,
}

#[allow(dead_code)]
enum EChannel<'a> {
    Translation(STranslationChannel<'a>),
    Rotation(SRotationChannel<'a>),
    Scale(SScaleChannel<'a>),
}

impl<'a> EChannel<'a> {
    fn node(&self) -> usize {
        match self {
            Self::Translation(tc) => {
                tc.node
            },
            Self::Rotation(rc) => {
                rc.node
            },
            Self::Scale(sc) => {
                sc.node
            },
        }
    }
}

fn find_segment_and_segment_t(time: f32, sample_times: &[f32]) -> (usize, f32) {
    assert!(sample_times.len() >= 2);
    let time = clamp(time, sample_times[0], *sample_times.last().expect("empty slice is no good"));

    for i in 0..(sample_times.len() - 1) {
        if sample_times[i] <= time && sample_times[i + 1] >= time {
            let segment_t = unlerp_f32(time, sample_times[i], sample_times[i + 1]);
            return (i, segment_t);
        }
    }

    panic!("It is expected to always return by here.");
}

impl<'a> STranslationChannel<'a> {
    pub fn sample(&self, time: f32) -> Vec3 {
        let (i, segment_t) = find_segment_and_segment_t(time, &self.sample_times);
        glm::lerp(&self.sample_values[i], &self.sample_values[i + 1], segment_t)
    }
}

impl<'a> SRotationChannel<'a> {
    pub fn sample(&self, time: f32) -> Quat {
        let (i, segment_t) = find_segment_and_segment_t(time, &self.sample_times);
        glm::quat_slerp(&self.sample_values[i], &self.sample_values[i + 1], segment_t)
    }
}

impl<'a> SScaleChannel<'a> {
    pub fn sample(&self, time: f32) -> f32 {
        let (i, segment_t) = find_segment_and_segment_t(time, &self.sample_times);
        lerp(self.sample_values[i], self.sample_values[i + 1], segment_t)
    }
}

pub fn update_joints<'a>(animation: &SAnimation, anim_time: f32, output_joints: &mut SMemVec<'a, STransform>) {
    for channel in animation.channels.as_ref() {
        let joint_idx = animation.bound_node_to_joint_map[channel.node()].expect("binding was bad!");

        match channel {
            EChannel::Translation(tc) => {
                output_joints[joint_idx].t = tc.sample(anim_time);
            },
            EChannel::Rotation(rc) => {
                output_joints[joint_idx].r = rc.sample(anim_time);
            },
            EChannel::Scale(sc) => {
                output_joints[joint_idx].s = sc.sample(anim_time);
            },
        }
    }
}

impl<'a> SAnimation<'a> {
    pub fn new_from_gltf(allocator: &'a dyn TMemAllocator, gltf_data: &gltf::Gltf, target_skinning: &SMeshSkinning) -> Result<Self, &'static str> {
        use gltf::animation::{Property};

        assert!(gltf_data.buffers().len() == 1, "can't handle multi-buffer gltf currently");
        let buffer = gltf_data.buffers().nth(0).unwrap();
        let buffer_bytes : Vec<u8> = {
            if let gltf::buffer::Source::Uri(binname) = buffer.source() {
                let path = std::path::Path::new("./assets/");
                let binname = std::path::Path::new(binname);
                let fullpath = path.join(binname);
                println!("Reading GLTF from path: {:?}", fullpath);
                std::fs::read(fullpath).unwrap()
            }
            else {
                panic!("Expected external buffer!");
            }
        };

        assert!(gltf_data.animations().len() == 1, "Can't handle multi-animation gltf currently");
        let animation = gltf_data.animations().nth(0).unwrap();

        let mut bound_node_to_joint_map = SMemVec::<Option<usize>>::new(allocator, gltf_data.nodes().len(), 0)?;
        bound_node_to_joint_map.push_all_default();

        let mut channels = SMemVec::<EChannel>::new(allocator, animation.channels().count(), 0)?;

        let mut duration = 0.0;

        for channel in animation.channels() {
            let target_node = channel.target().node();
            let target_node_idx = target_node.index();

            // -- bind the node idx to a joint
            {
                let target_node_name = target_node.name().expect("can't bind to skeleton without names");

                let joint_idx = target_skinning.joint_index_by_name(target_node_name).expect("failed to find joint idx corresponding to name");
                bound_node_to_joint_map[target_node_idx] = Some(joint_idx);
            }

            let sampler = channel.sampler();

            let sample_times_bin : &[f32] = gltf_accessor_slice(
                &sampler.input(),
                gltf::accessor::DataType::F32,
                gltf::accessor::Dimensions::Scalar,
                &buffer_bytes,
            );
            let sample_times = SMemVec::new_copy_slice(allocator, sample_times_bin)?;

            duration = f32::max(duration, *sample_times.last().unwrap());

            match channel.target().property() {
                Property::Translation => {
                    let sample_values_bin : &[Vec3] = gltf_accessor_slice(
                        &sampler.output(),
                        gltf::accessor::DataType::F32,
                        gltf::accessor::Dimensions::Vec3,
                        &buffer_bytes,
                    );
                    assert!(sample_times.len() == sample_values_bin.len());

                    channels.push(EChannel::Translation(
                        STranslationChannel{
                            node: target_node_idx,
                            sample_times,
                            sample_values: SMemVec::new_copy_slice(allocator, sample_values_bin)?,
                        }
                    ));
                },
                Property::Rotation => {
                    let sample_values_bin : &[Quat] = gltf_accessor_slice(
                        &sampler.output(),
                        gltf::accessor::DataType::F32,
                        gltf::accessor::Dimensions::Vec4,
                        &buffer_bytes,
                    );
                    assert!(sample_times.len() == sample_values_bin.len());

                    channels.push(EChannel::Rotation(
                        SRotationChannel{
                            node: target_node_idx,
                            sample_times,
                            sample_values: SMemVec::new_copy_slice(allocator, sample_values_bin)?,
                        }
                    ));
                },
                Property::Scale => {
                    // -- do nothing, not handling scale in animations currently
                }
                /*
                Property::Scale => {
                    let sample_values_bin : &[f32] = gltf_accessor_slice(
                        &sampler.output(),
                        gltf::accessor::DataType::F32,
                        gltf::accessor::Dimensions::Scalar,
                        &buffer_bytes,
                    );
                    assert!(sample_times.len() == sample_values_bin.len());

                    channels.push(EChannel::Scale(
                        SScaleChannel{
                            node: target_node_idx,
                            sample_times,
                            sample_values: SMemVec::new_copy_slice(allocator, sample_values_bin)?,
                        }
                    ));
                },*/
                _ => panic!("not implemented"),
            }
        }

        Ok(Self{
            duration,
            channels,
            bound_node_to_joint_map,
        })
    }
}
use allocate::{SAllocatorRef};
use collections::{SStoragePool, SPoolHandle, SVec};
use math::{Vec3, Quat};
use model::{SMeshSkinning};
use utils;
use utils::{STransform, lerp, unlerp_f32, gltf_accessor_slice, clamp};

pub struct SAnimation {
    pub duration: f32,
    channels: SVec<EChannel>,
    bound_node_to_joint_map: SVec<Option<usize>>,
}

struct STranslationChannel {
    node: usize,
    sample_times: SVec<f32>,
    sample_values: SVec<Vec3>,
}

struct SRotationChannel {
    node: usize,
    sample_times: SVec<f32>,
    sample_values: SVec<Quat>,
}

struct SScaleChannel {
    node: usize,
    sample_times: SVec<f32>,
    sample_values: SVec<f32>,
}

#[allow(dead_code)]
enum EChannel {
    Translation(STranslationChannel),
    Rotation(SRotationChannel),
    Scale(SScaleChannel),
}

impl EChannel {
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

pub struct SAnimLoaderEntry {
    uid: u64,
    animation: SAnimation,
}

pub struct SAnimationLoader {
    allocator: SAllocatorRef,
    animation_pool: SStoragePool<SAnimLoaderEntry, u16, u16>,
}
pub type SAnimHandle = SPoolHandle<u16, u16>;

fn find_segment_and_segment_t(time: f32, sample_times: &[f32]) -> (usize, f32) {
    assert!(sample_times.len() >= 2);
    let time = clamp(time, sample_times[0], *sample_times.last().expect("empty slice is no good"));

    for i in 0..(sample_times.len() - 1) {
        if sample_times[i] <= time && sample_times[i + 1] >= time {
            let segment_t = unlerp_f32(sample_times[i], sample_times[i + 1], time);
            return (i, segment_t);
        }
    }

    panic!("It is expected to always return by here.");
}

impl STranslationChannel {
    pub fn sample(&self, time: f32) -> Vec3 {
        let (i, segment_t) = find_segment_and_segment_t(time, &self.sample_times);
        Vec3::lerp(&self.sample_values[i], &self.sample_values[i + 1], segment_t)
    }
}

impl SRotationChannel {
    pub fn sample(&self, time: f32) -> Quat {
        let (i, segment_t) = find_segment_and_segment_t(time, &self.sample_times);
        Quat::slerp(&self.sample_values[i], &self.sample_values[i + 1], segment_t)
    }
}

impl SScaleChannel {
    pub fn sample(&self, time: f32) -> f32 {
        let (i, segment_t) = find_segment_and_segment_t(time, &self.sample_times);
        lerp(self.sample_values[i], self.sample_values[i + 1], segment_t)
    }
}

pub fn update_joints(animation: &SAnimation, anim_time: f32, output_joints: &mut SVec<STransform>) {
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

impl SAnimation {
    pub fn new_from_gltf(allocator: &SAllocatorRef, gltf_data: &gltf::Gltf, target_skinning: &SMeshSkinning) -> Result<Self, &'static str> {
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

        let mut bound_node_to_joint_map = SVec::<Option<usize>>::new(&allocator, gltf_data.nodes().len(), 0)?;
        bound_node_to_joint_map.push_all_default();

        let mut channels = SVec::<EChannel>::new(&allocator, animation.channels().count(), 0)?;

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
            let sample_times = SVec::new_copy_slice(&allocator, sample_times_bin)?;

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
                            sample_values: SVec::new_copy_slice(&allocator, sample_values_bin)?,
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

                    /*
                    println!("Dumping anim rotation properties:");
                    println!("sample_times: {:?}", sample_times.as_ref());
                    for sample_value in sample_values_bin {
                        println!("sample_values: {:?}", sample_value);
                    }
                    */

                    channels.push(EChannel::Rotation(
                        SRotationChannel{
                            node: target_node_idx,
                            sample_times,
                            sample_values: SVec::new_copy_slice(&allocator, sample_values_bin)?,
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
                            sample_values: SVec::new_copy_slice(allocator, sample_values_bin)?,
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

impl SAnimationLoader {
    pub fn new(allocator: SAllocatorRef, max_anim_count: usize) -> Self {
        Self{
            allocator,
            animation_pool: SStoragePool::create(max_anim_count as u16),
        }
    }

    fn find_anim_by_uid(&self, uid: u64) -> Option<SAnimHandle> {
        for i in 0..self.animation_pool.used() {
            if let Some(anim) = &self.animation_pool.get_by_index(i as u16).unwrap() {
                if anim.uid == uid {
                    return Some(self.animation_pool.handle_for_index(i as u16).expect("checked above"));
                }
            }
        }

        None
    }

    pub fn get_anim(&self, handle: SAnimHandle) -> Result<&SAnimation, &'static str> {
        self.animation_pool.get(handle).map(|entry| &entry.animation)
    }

    pub fn get_or_create_anim(&mut self, asset_file_path: &str, target_skinning: &SMeshSkinning) -> Result<SAnimHandle, &'static str> {
        assert!(asset_file_path.contains("assets/"));
        assert!(asset_file_path.contains(".gltf"));

        let uid = utils::hash_str(asset_file_path);
        if let Some(result) = self.find_anim_by_uid(uid) {
            return Ok(result);
        }

        let gltf_data = gltf::Gltf::open(asset_file_path).unwrap();
        let animation = SAnimation::new_from_gltf(&self.allocator, &gltf_data, target_skinning)?;

        let entry = SAnimLoaderEntry {
            uid,
            animation,
        };

        self.animation_pool.insert_val(entry)
    }
}
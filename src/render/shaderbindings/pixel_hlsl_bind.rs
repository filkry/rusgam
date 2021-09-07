use std::mem::{size_of};

use crate::niced3d12 as n12;
use crate::typeyd3d12 as t12;
use crate::math::{Vec4};
//use crate::model::{SMeshInstance};

pub struct SPixelHLSL {
    _bytecode: t12::SShaderBytecode,
}

pub struct SPixelHLSLBind {
    texture_metadata_rp_idx: usize,
    texture_rp_idx: usize,
    shadowcube_rp_idx: usize,
}

impl SPixelHLSL {
    // -- by convention, spaces 3-6 are for pixel shader use
    const BASESPACE: u32 = 3;

    pub fn new() -> Result<Self, &'static str> {
        let pixelblob = t12::read_file_to_blob("shaders_built/pixel.cso")?;
        let pixel_byte_code = t12::SShaderBytecode::create(pixelblob);

        Ok(Self{
            _bytecode: pixel_byte_code,
        })
    }

    pub fn bytecode(&self) -> &t12::SShaderBytecode {
        &self._bytecode
    }

    pub fn bind(&self, root_signature_desc: &mut t12::SRootSignatureDesc, texture_array_size: u32) -> SPixelHLSLBind {
        let mut add_param = |param: n12::SRootParameter| -> usize {
            root_signature_desc.parameters.push(param.into_raw());
            root_signature_desc.parameters.len() - 1
        };

        let texture_metadata_rp_idx = add_param(n12::SRootParameter::new_srv_descriptor(
            Self::TEXTUREMETADATAREGISTER,
            Self::BASESPACE,
            t12::EShaderVisibility::Pixel,
        ));

        let textures_rp_idx = add_param(n12::SRootParameter::new_unique_space_srv_descriptor_table(
            Self::TEXTURESPACE,
            t12::EShaderVisibility::Pixel,
            texture_array_size,
        ));

        let shadow_cube_root_parameter = {
            let descriptor_range = t12::SDescriptorRange {
                range_type: t12::EDescriptorRangeType::SRV,
                num_descriptors: 1,
                base_shader_register: 0,
                register_space: Self::SHADOWSPACE,
                offset_in_descriptors_from_table_start: t12::EDescriptorRangeOffset::EAppend,
            };

            let mut root_descriptor_table = t12::SRootDescriptorTable::new();
            root_descriptor_table
                .descriptor_ranges
                .push(descriptor_range);

            t12::SRootParameter {
                type_: t12::ERootParameterType::DescriptorTable(root_descriptor_table),
                shader_visibility: t12::EShaderVisibility::Pixel,
            }
        };

        let sampler = t12::SStaticSamplerDesc {
            filter: t12::EFilter::MinMagMipPoint,
            address_u: t12::ETextureAddressMode::Border,
            address_v: t12::ETextureAddressMode::Border,
            address_w: t12::ETextureAddressMode::Border,
            mip_lod_bias: 0.0,
            max_anisotropy: 0,
            comparison_func: t12::EComparisonFunc::Never,
            border_color: t12::EStaticBorderColor::OpaqueWhite,
            min_lod: 0.0,
            max_lod: std::f32::MAX,
            shader_register: 0,
            register_space: Self::BASESPACE,
            shader_visibility: t12::EShaderVisibility::Pixel,
        };

        let shadow_sampler = t12::SStaticSamplerDesc {
            filter: t12::EFilter::MinMagMipPoint,
            address_u: t12::ETextureAddressMode::Clamp,
            address_v: t12::ETextureAddressMode::Clamp,
            address_w: t12::ETextureAddressMode::Clamp,
            mip_lod_bias: 0.0,
            max_anisotropy: 0,
            comparison_func: t12::EComparisonFunc::Never,
            border_color: t12::EStaticBorderColor::OpaqueWhite,
            min_lod: 0.0,
            max_lod: 0.0,
            shader_register: 0,
            register_space: Self::SHADOWSPACE,
            shader_visibility: t12::EShaderVisibility::Pixel,
        };

        root_signature_desc.parameters.push(texture_metadata_root_parameter);
        let texture_metadata_rp_idx = root_signature_desc.parameters.len() - 1;
        root_signature_desc.parameters.push(texture_root_parameter);
        let texture_rp_idx = root_signature_desc.parameters.len() - 1;
        root_signature_desc.parameters.push(shadow_cube_root_parameter);
        let shadowcube_rp_idx = root_signature_desc.parameters.len() - 1;

        root_signature_desc.static_samplers.push(sampler);
        root_signature_desc.static_samplers.push(shadow_sampler);

        SPixelHLSLBind {
            texture_metadata_rp_idx,
            texture_rp_idx,
            shadowcube_rp_idx,
        }
    }

    pub fn set_graphics_roots(
        &self,
        bind: &SPixelHLSLBind,
        list: &mut n12::SCommandList,
        texture_metadata: STextureMetadata,
        texture_gpu_descriptor: Option<t12::SGPUDescriptorHandle>,
        shadowcube_gpu_descriptor: t12::SGPUDescriptorHandle)
    {
        list.set_graphics_root_descriptor_table(bind.shadowcube_rp_idx, &shadowcube_gpu_descriptor);
        list.set_graphics_root_32_bit_constants(bind.texture_metadata_rp_idx, &texture_metadata, 0);
        if let Some(t) = texture_gpu_descriptor {
            assert!(texture_metadata.has_diffuse_texture == 1.0);
            list.set_graphics_root_descriptor_table(bind.texture_rp_idx, &t);
        }
    }
}
use super::*;

pub struct SRootSignature {
    raw: t12::SRootSignature,
    desc: t12::SRootSignatureDesc,

    // -- intermediate data
    serialized_blob: t12::SBlob,
}

pub struct SRootParameter {
    raw: t12::SRootParameter,
}

impl SDevice {
    pub fn create_root_signature(
        &self,
        mut desc: t12::SRootSignatureDesc,
        version: t12::ERootSignatureVersion,
    ) -> Result<SRootSignature, &'static str> {
        let serialized_blob = t12::serialize_root_signature(&mut desc, version)
            .ok()
            .expect("Could not serialize root signature");

        let rs = self.raw().create_root_signature(&serialized_blob)?;

        Ok(SRootSignature {
            raw: rs,
            desc: desc,
            serialized_blob: serialized_blob,
        })
    }
}

impl SRootSignature {
    pub fn raw(&self) -> &t12::SRootSignature {
        &self.raw
    }

    pub fn desc(&self) -> &t12::SRootSignatureDesc {
        &self.desc
    }
}

impl SRootParameter {
    pub fn raw(&self) -> &t12::SRootParameter {
        &self.raw
    }

    pub fn into_raw(self) -> t12::SRootParameter {
        self.raw
    }

    pub fn new_srv_descriptor(register: u32, space: u32, visibility: t12::EShaderVisibility) -> Self {
        let raw = t12::SRootParameter {
            type_: t12::ERootParameterType::SRV(
                t12::SRootDescriptor {
                    shader_register: register,
                    register_space: space,
                }),
            shader_visibility: visibility,
        };
        Self {
            raw,
        }
    }

    pub fn new_uav_descriptor(register: u32, space: u32, visibility: t12::EShaderVisibility) -> Self {
        let raw = t12::SRootParameter {
            type_: t12::ERootParameterType::UAV(
                t12::SRootDescriptor {
                    shader_register: register,
                    register_space: space,
                }),
            shader_visibility: visibility,
        };
        Self {
            raw,
        }
    }
}
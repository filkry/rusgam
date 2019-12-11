use super::*;

pub struct SRootSignature {
    raw: t12::SRootSignature,
    desc: t12::SRootSignatureDesc,

    // -- intermediate data
    serialized_blob: t12::SBlob,
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

use super::*;

#[derive(Copy, Clone, PartialEq)]
pub enum ERootSignatureFlags {
    ENone,
    AllowInputAssemblerInputLayout,
    DenyVertexShaderRootAccess,
    DenyHullShaderRootAccess,
    DenyDomainShaderRootAccess,
    DenyGeometryShaderRootAccess,
    DenyPixelShaderRootAccess,
    AllowStreamOutput,
    //LocalRootSignature,
}

impl TEnumFlags for ERootSignatureFlags {
    type TRawType = win::D3D12_ROOT_SIGNATURE_FLAGS;

    fn rawtype(&self) -> Self::TRawType {
        match self {
            Self::ENone => win::D3D12_ROOT_SIGNATURE_FLAG_NONE,
            Self::AllowInputAssemblerInputLayout => {
                win::D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT
            }
            Self::DenyVertexShaderRootAccess => {
                win::D3D12_ROOT_SIGNATURE_FLAG_DENY_VERTEX_SHADER_ROOT_ACCESS
            }
            Self::DenyHullShaderRootAccess => {
                win::D3D12_ROOT_SIGNATURE_FLAG_DENY_HULL_SHADER_ROOT_ACCESS
            }
            Self::DenyDomainShaderRootAccess => {
                win::D3D12_ROOT_SIGNATURE_FLAG_DENY_DOMAIN_SHADER_ROOT_ACCESS
            }
            Self::DenyGeometryShaderRootAccess => {
                win::D3D12_ROOT_SIGNATURE_FLAG_DENY_GEOMETRY_SHADER_ROOT_ACCESS
            }
            Self::DenyPixelShaderRootAccess => {
                win::D3D12_ROOT_SIGNATURE_FLAG_DENY_PIXEL_SHADER_ROOT_ACCESS
            }
            Self::AllowStreamOutput => win::D3D12_ROOT_SIGNATURE_FLAG_ALLOW_STREAM_OUTPUT,
            //Self::LocalRootSignature => D3D12_ROOT_SIGNATURE_FLAG_LOCAL_ROOT_SIGNATURE
        }
    }
}

pub type SRootSignatureFlags = SEnumFlags<ERootSignatureFlags>;

#[derive(Clone)]
pub struct SRootSignature {
    pub raw: win::ID3D12RootSignature,
}

// -- $$$FRK(TODO): This struct should take references to slices for parameters/static_samplers,
// -- and we also need a better solution to this recurring d3dtype problem
pub struct SRootSignatureDesc {
    pub parameters: Vec<SRootParameter>,
    pub static_samplers: Vec<SStaticSamplerDesc>,
    pub flags: SRootSignatureFlags,

    // -- for d3dtype()
    d3d_parameters: Vec<win::D3D12_ROOT_PARAMETER>,
    d3d_static_samplers: Vec<win::D3D12_STATIC_SAMPLER_DESC>,
}

impl SRootSignatureDesc {
    pub fn new(flags: SRootSignatureFlags) -> Self {
        Self {
            parameters: Vec::new(),
            static_samplers: Vec::new(),
            flags: flags,
            d3d_parameters: Vec::new(),
            d3d_static_samplers: Vec::new(),
        }
    }

    pub unsafe fn d3dtype(&mut self) -> win::D3D12_ROOT_SIGNATURE_DESC {
        self.d3d_parameters.clear();
        for parameter in &mut self.parameters {
            self.d3d_parameters.push(parameter.d3dtype());
        }

        self.d3d_static_samplers.clear();
        for sampler in &self.static_samplers {
            self.d3d_static_samplers.push(sampler.d3dtype());
        }

        win::D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: self.parameters.len() as u32,
            pParameters: self.d3d_parameters.as_mut_ptr(),
            NumStaticSamplers: self.static_samplers.len() as u32,
            pStaticSamplers: self.d3d_static_samplers.as_mut_ptr(),
            Flags: self.flags.rawtype(),
        }
    }
}

pub enum ERootSignatureVersion {
    V1,
    V1_0,
    V1_1,
}

impl ERootSignatureVersion {
    pub fn d3dtype(&self) -> win::D3D_ROOT_SIGNATURE_VERSION {
        match self {
            Self::V1 => win::D3D_ROOT_SIGNATURE_VERSION_1,
            Self::V1_0 => win::D3D_ROOT_SIGNATURE_VERSION_1_0,
            Self::V1_1 => win::D3D_ROOT_SIGNATURE_VERSION_1_1,
        }
    }
}

pub fn serialize_root_signature(
    root_signature: &mut SRootSignatureDesc,
    version: ERootSignatureVersion,
) -> Result<SBlob, SBlob> {
    let mut raw_result_blob: Option<win::ID3DBlob> = None;
    let mut raw_err_blob: Option<win::ID3DBlob> = None;

    let d3d_signature = unsafe { root_signature.d3dtype() };

    let hr = unsafe {
        win::D3D12SerializeRootSignature(
            &d3d_signature,
            version.d3dtype(),
            &mut raw_result_blob,
            &mut raw_err_blob,
        )
    };

    match hr {
        Ok(_) => Ok(SBlob { raw: raw_result_blob.expect("") }),
        Err(_) => Err(SBlob { raw: raw_err_blob.expect("") }),
    }
}

pub struct SRootConstants {
    pub shader_register: u32,
    pub register_space: u32,
    pub num_32_bit_values: u32,
}

impl SRootConstants {
    pub fn d3dtype(&self) -> win::D3D12_ROOT_CONSTANTS {
        win::D3D12_ROOT_CONSTANTS {
            ShaderRegister: self.shader_register,
            RegisterSpace: self.register_space,
            Num32BitValues: self.num_32_bit_values,
        }
    }
}

pub struct SRootDescriptorTable {
    pub descriptor_ranges: ArrayVec<[SDescriptorRange; 16]>,

    // -- for d3dtype()
    d3d_descriptor_ranges: ArrayVec<[win::D3D12_DESCRIPTOR_RANGE; 16]>,
}

impl SRootDescriptorTable {
    pub fn new() -> Self {
        Self {
            descriptor_ranges: ArrayVec::new(),
            d3d_descriptor_ranges: ArrayVec::new(),
        }
    }

    pub unsafe fn d3dtype(&mut self) -> win::D3D12_ROOT_DESCRIPTOR_TABLE {
        self.d3d_descriptor_ranges.clear();
        for dr in &self.descriptor_ranges {
            self.d3d_descriptor_ranges.push(dr.d3dtype());
        }

        win::D3D12_ROOT_DESCRIPTOR_TABLE {
            NumDescriptorRanges: self.d3d_descriptor_ranges.len() as u32,
            pDescriptorRanges: self.d3d_descriptor_ranges.as_mut_ptr(),
        }
    }
}

pub struct SRootDescriptor {
    pub shader_register: u32,
    pub register_space: u32,
}

impl SRootDescriptor {
    pub fn d3dtype(&self) -> win::D3D12_ROOT_DESCRIPTOR {
        win::D3D12_ROOT_DESCRIPTOR {
            ShaderRegister: self.shader_register,
            RegisterSpace: self.register_space,
        }
    }
}

pub enum ERootParameterType {
    DescriptorTable(SRootDescriptorTable),
    E32BitConstants(SRootConstants),
    CBV(SRootDescriptor),
    SRV(SRootDescriptor),
    UAV(SRootDescriptor),
}

impl ERootParameterType {
    pub fn d3dtype(&self) -> win::D3D12_ROOT_PARAMETER_TYPE {
        match self {
            Self::DescriptorTable(..) => win::D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            Self::E32BitConstants(..) => win::D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
            Self::CBV(..) => win::D3D12_ROOT_PARAMETER_TYPE_CBV,
            Self::SRV(..) => win::D3D12_ROOT_PARAMETER_TYPE_SRV,
            Self::UAV(..) => win::D3D12_ROOT_PARAMETER_TYPE_UAV,
        }
    }
}

pub struct SRootParameter {
    pub type_: ERootParameterType,
    pub shader_visibility: EShaderVisibility,
}

impl SRootParameter {
    pub fn d3dtype(&mut self) -> win::D3D12_ROOT_PARAMETER {
        unsafe {
            let mut result = mem::MaybeUninit::<win::D3D12_ROOT_PARAMETER>::zeroed().assume_init();
            result.ParameterType = self.type_.d3dtype();
            match &mut self.type_ {
                ERootParameterType::E32BitConstants ( constants ) => {
                    result.Anonymous.Constants = constants.d3dtype();
                }
                ERootParameterType::DescriptorTable ( table ) => {
                    result.Anonymous.DescriptorTable = table.d3dtype();
                }
                ERootParameterType::CBV ( descriptor ) => {
                    result.Anonymous.Descriptor = descriptor.d3dtype();
                }
                ERootParameterType::SRV ( descriptor ) => {
                    result.Anonymous.Descriptor = descriptor.d3dtype();
                }
                ERootParameterType::UAV ( descriptor ) => {
                    result.Anonymous.Descriptor = descriptor.d3dtype();
                }
            }
            result.ShaderVisibility = self.shader_visibility.d3dtype();

            result
        }
    }
}

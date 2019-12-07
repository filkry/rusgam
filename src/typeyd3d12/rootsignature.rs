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

impl TD3DFlags32 for ERootSignatureFlags {
    type TD3DType = u32;

    fn d3dtype(&self) -> Self::TD3DType {
        match self {
            Self::ENone => D3D12_ROOT_SIGNATURE_FLAG_NONE,
            Self::AllowInputAssemblerInputLayout => {
                D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT
            }
            Self::DenyVertexShaderRootAccess => {
                D3D12_ROOT_SIGNATURE_FLAG_DENY_VERTEX_SHADER_ROOT_ACCESS
            }
            Self::DenyHullShaderRootAccess => {
                D3D12_ROOT_SIGNATURE_FLAG_DENY_HULL_SHADER_ROOT_ACCESS
            }
            Self::DenyDomainShaderRootAccess => {
                D3D12_ROOT_SIGNATURE_FLAG_DENY_DOMAIN_SHADER_ROOT_ACCESS
            }
            Self::DenyGeometryShaderRootAccess => {
                D3D12_ROOT_SIGNATURE_FLAG_DENY_GEOMETRY_SHADER_ROOT_ACCESS
            }
            Self::DenyPixelShaderRootAccess => {
                D3D12_ROOT_SIGNATURE_FLAG_DENY_PIXEL_SHADER_ROOT_ACCESS
            }
            Self::AllowStreamOutput => D3D12_ROOT_SIGNATURE_FLAG_ALLOW_STREAM_OUTPUT,
            //Self::LocalRootSignature => D3D12_ROOT_SIGNATURE_FLAG_LOCAL_ROOT_SIGNATURE
        }
    }
}

pub type SRootSignatureFlags = SD3DFlags32<ERootSignatureFlags>;

pub struct SRootSignature {
    pub raw: ComPtr<ID3D12RootSignature>,
}

pub struct SRootSignatureDesc {
    pub parameters: Vec<SRootParameter>,
    //static_samplers: Vec<SStaticSamplerDesc>,
    pub flags: SRootSignatureFlags,

    // -- for d3dtype()
    d3d_parameters: Vec<D3D12_ROOT_PARAMETER>,
}

impl SRootSignatureDesc {
    pub fn new(flags: SRootSignatureFlags) -> Self {
        Self {
            parameters: Vec::new(), // $$$FRK(TODO): allocations
            flags: flags,
            d3d_parameters: Vec::new(),
        }
    }

    pub unsafe fn d3dtype(&mut self) -> D3D12_ROOT_SIGNATURE_DESC {
        self.d3d_parameters.clear();
        for parameter in &self.parameters {
            self.d3d_parameters.push(parameter.d3dtype());
        }

        D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: self.parameters.len() as u32,
            pParameters: self.d3d_parameters.as_ptr(),
            NumStaticSamplers: 0,
            pStaticSamplers: ptr::null_mut(),
            Flags: self.flags.d3dtype(),
        }
    }
}

pub enum ERootSignatureVersion {
    V1,
    V1_0,
    V1_1,
}

impl ERootSignatureVersion {
    pub fn d3dtype(&self) -> D3D_ROOT_SIGNATURE_VERSION {
        match self {
            Self::V1 => D3D_ROOT_SIGNATURE_VERSION_1,
            Self::V1_0 => D3D_ROOT_SIGNATURE_VERSION_1_0,
            Self::V1_1 => D3D_ROOT_SIGNATURE_VERSION_1_1,
        }
    }
}

pub fn serialize_root_signature(
    root_signature: &mut SRootSignatureDesc,
    version: ERootSignatureVersion,
) -> Result<SBlob, SBlob> {
    let mut raw_result_blob: *mut d3dcommon::ID3DBlob = ptr::null_mut();
    let mut raw_err_blob: *mut d3dcommon::ID3DBlob = ptr::null_mut();

    let d3d_signature = unsafe { root_signature.d3dtype() };

    let hr = unsafe {
        D3D12SerializeRootSignature(
            &d3d_signature,
            version.d3dtype(),
            &mut raw_result_blob,
            &mut raw_err_blob,
        )
    };

    if winerror::SUCCEEDED(hr) {
        Ok(SBlob {
            raw: unsafe { ComPtr::from_raw(raw_result_blob) },
        })
    } else {
        Err(SBlob {
            raw: unsafe { ComPtr::from_raw(raw_err_blob) },
        })
    }
}

pub struct SRootConstants {
    pub shader_register: u32,
    pub register_space: u32,
    pub num_32_bit_values: u32,
}

impl SRootConstants {
    pub fn d3dtype(&self) -> D3D12_ROOT_CONSTANTS {
        D3D12_ROOT_CONSTANTS {
            ShaderRegister: self.shader_register,
            RegisterSpace: self.register_space,
            Num32BitValues: self.num_32_bit_values,
        }
    }
}

pub enum ERootParameterType {
    DescriptorTable,
    E32BitConstants,
    CBV,
    SRV,
    UAV,
}

impl ERootParameterType {
    pub fn d3dtype(&self) -> D3D12_ROOT_PARAMETER_TYPE {
        match self {
            Self::DescriptorTable => D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            Self::E32BitConstants => D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
            Self::CBV => D3D12_ROOT_PARAMETER_TYPE_CBV,
            Self::SRV => D3D12_ROOT_PARAMETER_TYPE_SRV,
            Self::UAV => D3D12_ROOT_PARAMETER_TYPE_UAV,
        }
    }
}

pub enum ERootParameterTypeData {
    Constants { constants: SRootConstants },
}

pub struct SRootParameter {
    pub type_: ERootParameterType,
    pub type_data: ERootParameterTypeData,
    pub shader_visibility: EShaderVisibility,
}

impl SRootParameter {
    pub fn d3dtype(&self) -> D3D12_ROOT_PARAMETER {
        unsafe {
            let mut result = mem::MaybeUninit::<D3D12_ROOT_PARAMETER>::zeroed();
            result.ParameterType = self.type_.d3dtype();
            match &self.type_data {
                ERootParameterTypeData::Constants { constants } => {
                    *result.u.Constants_mut() = constants.d3dtype();
                }
            }
            result.ShaderVisibility = self.shader_visibility.d3dtype();

            result
        }
    }
}

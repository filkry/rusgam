use super::*;

#[derive(Copy, Clone, PartialEq)]
pub enum ECompile {
    Debug,
    SkipValidation,
    SkipOptimization,
    PackMatrixRowMajor,
    PackMatrixColumnMajor,
    PartialPrecision,
    ForceVsSoftwareNoOpt,
    ForcePsSoftwareNoOpt,
    NoPreshader,
    AvoidFlowControl,
    PreferFlowControl,
    EnableStrictness,
    EnableBackwardsCompatibility,
    IEEEStrictness,
    OptimizationLevel0,
    OptimizationLevel1,
    OptimizationLevel2,
    OptimizationLevel3,
    WarningsAreErrors,
    ResourcesMayAlias,
    //EnableUnboundedDescriptorTables,
    AllResourcesBound,
}

impl TEnumFlags32 for ECompile {
    type TRawType = DWORD;

    fn rawtype(&self) -> Self::TRawType {
        match self {
            ECompile::Debug => d3dcompiler::D3DCOMPILE_DEBUG,
            ECompile::SkipValidation => d3dcompiler::D3DCOMPILE_SKIP_VALIDATION,
            ECompile::SkipOptimization => d3dcompiler::D3DCOMPILE_SKIP_OPTIMIZATION,
            ECompile::PackMatrixRowMajor => d3dcompiler::D3DCOMPILE_PACK_MATRIX_ROW_MAJOR,
            ECompile::PackMatrixColumnMajor => d3dcompiler::D3DCOMPILE_PACK_MATRIX_COLUMN_MAJOR,
            ECompile::PartialPrecision => d3dcompiler::D3DCOMPILE_PARTIAL_PRECISION,
            ECompile::ForceVsSoftwareNoOpt => d3dcompiler::D3DCOMPILE_FORCE_VS_SOFTWARE_NO_OPT,
            ECompile::ForcePsSoftwareNoOpt => d3dcompiler::D3DCOMPILE_FORCE_PS_SOFTWARE_NO_OPT,
            ECompile::NoPreshader => d3dcompiler::D3DCOMPILE_NO_PRESHADER,
            ECompile::AvoidFlowControl => d3dcompiler::D3DCOMPILE_AVOID_FLOW_CONTROL,
            ECompile::PreferFlowControl => d3dcompiler::D3DCOMPILE_PREFER_FLOW_CONTROL,
            ECompile::EnableStrictness => d3dcompiler::D3DCOMPILE_ENABLE_STRICTNESS,
            ECompile::EnableBackwardsCompatibility => {
                d3dcompiler::D3DCOMPILE_ENABLE_BACKWARDS_COMPATIBILITY
            }
            ECompile::IEEEStrictness => d3dcompiler::D3DCOMPILE_IEEE_STRICTNESS,
            ECompile::OptimizationLevel0 => d3dcompiler::D3DCOMPILE_OPTIMIZATION_LEVEL0,
            ECompile::OptimizationLevel1 => d3dcompiler::D3DCOMPILE_OPTIMIZATION_LEVEL1,
            ECompile::OptimizationLevel2 => d3dcompiler::D3DCOMPILE_OPTIMIZATION_LEVEL2,
            ECompile::OptimizationLevel3 => d3dcompiler::D3DCOMPILE_OPTIMIZATION_LEVEL3,
            ECompile::WarningsAreErrors => d3dcompiler::D3DCOMPILE_WARNINGS_ARE_ERRORS,
            ECompile::ResourcesMayAlias => d3dcompiler::D3DCOMPILE_RESOURCES_MAY_ALIAS,
            //ECompile::EnableUnboundedDescriptorTables => d3dcompiler::D3DCOMPILE_ENABLE_UNBOUND_DESCRIPTOR_TABLES,
            ECompile::AllResourcesBound => d3dcompiler::D3DCOMPILE_ALL_RESOURCES_BOUND,
        }
    }
}

pub type SCompile = SEnumFlags32<ECompile>;

pub enum EShaderVisibility {
    All,
    Vertex,
    Hull,
    Domain,
    Geometry,
    Pixel,
}

impl EShaderVisibility {
    pub fn d3dtype(&self) -> D3D12_SHADER_VISIBILITY {
        match self {
            Self::All => D3D12_SHADER_VISIBILITY_ALL,
            Self::Vertex => D3D12_SHADER_VISIBILITY_VERTEX,
            Self::Hull => D3D12_SHADER_VISIBILITY_HULL,
            Self::Domain => D3D12_SHADER_VISIBILITY_DOMAIN,
            Self::Geometry => D3D12_SHADER_VISIBILITY_GEOMETRY,
            Self::Pixel => D3D12_SHADER_VISIBILITY_PIXEL,
        }
    }
}

pub struct SShaderBytecode {
    bytecode: SBlob,
}

impl SShaderBytecode {
    pub fn create(blob: SBlob) -> Self {
        Self { bytecode: blob }
    }

    pub unsafe fn d3dtype(&self) -> D3D12_SHADER_BYTECODE {
        let ptr = self.bytecode.raw.GetBufferPointer();
        let len = self.bytecode.raw.GetBufferSize();

        D3D12_SHADER_BYTECODE {
            pShaderBytecode: ptr,
            BytecodeLength: len,
        }
    }
}

// -- $$$FRK(FUTURE WORK): unsupported:
// --    + pDefines
// --    + pInclude
// --    + flags2
pub fn d3dcompilefromfile(
    file: &str,
    entrypoint: &str,
    target: &str,
    flags1: SCompile,
) -> Result<SBlob, &'static str> {
    // -- $$$FRK(FUTURE WORK): allocations :(
    let mut fileparam: Vec<u16> = file.encode_utf16().collect();
    fileparam.push('\0' as u16);

    let mut entrypointparam: Vec<char> = entrypoint.chars().collect();
    entrypointparam.push('\0');

    let mut targetparam: Vec<char> = target.chars().collect();
    targetparam.push('\0');

    let mut rawcodeblob: *mut d3dcommon::ID3DBlob = ptr::null_mut();
    let mut errormsgsblob: *mut d3dcommon::ID3DBlob = ptr::null_mut();

    let hr = unsafe {
        d3dcompiler::D3DCompileFromFile(
            fileparam.as_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            entrypointparam.as_ptr() as *const i8,
            targetparam.as_ptr() as *const i8,
            flags1.rawtype(),
            0,
            &mut rawcodeblob,
            &mut errormsgsblob,
        )
    };

    returnerrifwinerror!(hr, "failed to compile shader");
    // -- $$$FRK(TODO): use error messages blob

    Ok(SBlob {
        raw: unsafe { ComPtr::from_raw(rawcodeblob) },
    })
}

pub fn read_file_to_blob(file: &str) -> Result<SBlob, &'static str> {
    let mut fileparam: Vec<u16> = file.encode_utf16().collect();
    fileparam.push('\0' as u16);

    let mut resultblob: *mut d3dcommon::ID3DBlob = ptr::null_mut();

    let hr = unsafe { d3dcompiler::D3DReadFileToBlob(fileparam.as_ptr(), &mut resultblob) };

    returnerrifwinerror!(hr, "failed to load shader");

    Ok(SBlob {
        raw: unsafe { ComPtr::from_raw(resultblob) },
    })
}

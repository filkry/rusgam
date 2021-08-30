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
    type TRawType = u32;

    fn rawtype(&self) -> Self::TRawType {
        match self {
            ECompile::Debug => win::D3DCOMPILE_DEBUG,
            ECompile::SkipValidation => win::D3DCOMPILE_SKIP_VALIDATION,
            ECompile::SkipOptimization => win::D3DCOMPILE_SKIP_OPTIMIZATION,
            ECompile::PackMatrixRowMajor => win::D3DCOMPILE_PACK_MATRIX_ROW_MAJOR,
            ECompile::PackMatrixColumnMajor => win::D3DCOMPILE_PACK_MATRIX_COLUMN_MAJOR,
            ECompile::PartialPrecision => win::D3DCOMPILE_PARTIAL_PRECISION,
            ECompile::ForceVsSoftwareNoOpt => win::D3DCOMPILE_FORCE_VS_SOFTWARE_NO_OPT,
            ECompile::ForcePsSoftwareNoOpt => win::D3DCOMPILE_FORCE_PS_SOFTWARE_NO_OPT,
            ECompile::NoPreshader => win::D3DCOMPILE_NO_PRESHADER,
            ECompile::AvoidFlowControl => win::D3DCOMPILE_AVOID_FLOW_CONTROL,
            ECompile::PreferFlowControl => win::D3DCOMPILE_PREFER_FLOW_CONTROL,
            ECompile::EnableStrictness => win::D3DCOMPILE_ENABLE_STRICTNESS,
            ECompile::EnableBackwardsCompatibility => {
                win::D3DCOMPILE_ENABLE_BACKWARDS_COMPATIBILITY
            }
            ECompile::IEEEStrictness => win::D3DCOMPILE_IEEE_STRICTNESS,
            ECompile::OptimizationLevel0 => win::D3DCOMPILE_OPTIMIZATION_LEVEL0,
            ECompile::OptimizationLevel1 => win::D3DCOMPILE_OPTIMIZATION_LEVEL1,
            ECompile::OptimizationLevel2 => win::D3DCOMPILE_OPTIMIZATION_LEVEL2,
            ECompile::OptimizationLevel3 => win::D3DCOMPILE_OPTIMIZATION_LEVEL3,
            ECompile::WarningsAreErrors => win::D3DCOMPILE_WARNINGS_ARE_ERRORS,
            ECompile::ResourcesMayAlias => win::D3DCOMPILE_RESOURCES_MAY_ALIAS,
            //ECompile::EnableUnboundedDescriptorTables => d3dcompiler::D3DCOMPILE_ENABLE_UNBOUND_DESCRIPTOR_TABLES,
            ECompile::AllResourcesBound => win::D3DCOMPILE_ALL_RESOURCES_BOUND,
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
    pub fn d3dtype(&self) -> win::D3D12_SHADER_VISIBILITY {
        match self {
            Self::All => win::D3D12_SHADER_VISIBILITY_ALL,
            Self::Vertex => win::D3D12_SHADER_VISIBILITY_VERTEX,
            Self::Hull => win::D3D12_SHADER_VISIBILITY_HULL,
            Self::Domain => win::D3D12_SHADER_VISIBILITY_DOMAIN,
            Self::Geometry => win::D3D12_SHADER_VISIBILITY_GEOMETRY,
            Self::Pixel => win::D3D12_SHADER_VISIBILITY_PIXEL,
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

    pub unsafe fn d3dtype(&self) -> win::D3D12_SHADER_BYTECODE {
        let ptr = self.bytecode.raw.GetBufferPointer();
        let len = self.bytecode.raw.GetBufferSize();

        win::D3D12_SHADER_BYTECODE {
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

    let mut rawcodeblob: Option<win::ID3DBlob> = None;
    let mut errormsgsblob: Option<win::ID3DBlob> = None;

    let hr = unsafe {
        win::D3DCompileFromFile(
            file,
            ptr::null_mut(),
            None,
            entrypoint,
            target,
            flags1.rawtype(),
            0,
            &mut rawcodeblob,
            &mut errormsgsblob,
        )
    };

    returnerrifwinerror!(hr, "failed to compile shader");
    // -- $$$FRK(TODO): use error messages blob

    Ok(SBlob {
        raw: rawcodeblob.expect("checked err above"),
    })
}

pub fn read_file_to_blob(file: &str) -> Result<SBlob, &'static str> {
    let hr = unsafe { win::D3DReadFileToBlob(file) };

    returnerrifwinerror!(hr, "failed to load shader");

    Ok(SBlob {
        raw: hr.expect("checked err above"),
    })
}

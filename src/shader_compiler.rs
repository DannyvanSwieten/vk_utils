use byteorder::ReadBytesExt;
use rspirv_reflect::{DescriptorInfo, Reflection};
use std::path::Path;
use std::{collections::BTreeMap, fs::File};

use shaderc::{CompilationArtifact, CompileOptions, Compiler, OptimizationLevel, ShaderKind};

pub fn load_spirv(path: &str) -> Vec<u32> {
    let file = File::open(path).expect(&(String::from("File not found at: ") + path));
    let meta = std::fs::metadata(path).expect("No metadata found for file");
    let mut buf_reader = std::io::BufReader::new(file);

    let mut buffer = vec![0; (meta.len() / 4) as usize];
    buf_reader
        .read_u32_into::<byteorder::NativeEndian>(&mut buffer[..])
        .expect("Failed reading spirv");

    buffer
}

pub struct ShaderReflection {
    // module: ShaderModule,
    reflection: Reflection,
}

impl ShaderReflection {
    pub fn descriptor_sets(&self) -> Option<BTreeMap<u32, BTreeMap<u32, DescriptorInfo>>> {
        match self.reflection.get_descriptor_sets() {
            Ok(sets) => Some(sets),
            Err(_) => None,
        }
    }

    pub fn push_constant_ranges(
        &self,
    ) -> Result<Option<rspirv_reflect::PushConstantInfo>, rspirv_reflect::ReflectError> {
        self.reflection.get_push_constant_range()
    }
}

pub struct CompilationResult {
    result: Result<CompilationArtifact, shaderc::Error>,
}

impl CompilationResult {
    pub fn failed(&self) -> bool {
        self.result.is_err()
    }

    pub fn error_string(&self) -> String {
        match &self.result {
            Ok(_) => "".to_owned(),
            Err(e) => e.to_string(),
        }
    }

    pub fn has_warnings(&self) -> bool {
        match &self.result {
            Ok(s) => s.get_num_warnings() > 0,
            Err(_) => false,
        }
    }

    pub fn spirv(&self) -> &[u32] {
        match &self.result {
            Ok(s) => s.as_binary(),
            Err(_) => panic!(),
        }
    }

    pub fn reflect(&self) -> ShaderReflection {
        match &self.result {
            Ok(s) => match Reflection::new_from_spirv(s.as_binary_u8()) {
                Ok(reflection) => ShaderReflection { reflection },
                Err(e) => panic!("Error: {}", e),
            },
            Err(e) => panic!("Error: {}", e),
        }
    }
}
pub struct ShaderCompiler {}

impl ShaderCompiler {
    pub fn compile_file(
        path: &Path,
        kind: ShaderKind,
        entry_point: &str,
    ) -> Option<CompilationResult> {
        let src = match std::fs::read_to_string(path) {
            Ok(text) => Some(text),
            Err(_) => None,
        };

        if let Some(src) = src {
            let result = Self::compile_string(&src, kind, path.to_str().unwrap(), entry_point);
            Some(result)
        } else {
            None
        }
    }

    pub fn compile_string(
        src: &str,
        kind: ShaderKind,
        origin: &str,
        entry_point: &str,
    ) -> CompilationResult {
        let compiler = Compiler::new();
        if let Some(compiler) = compiler {
            let mut options = CompileOptions::new().unwrap();
            options.set_target_spirv(shaderc::SpirvVersion::V1_6);
            options.set_optimization_level(OptimizationLevel::Performance);
            let result = compiler.compile_into_spirv(src, kind, origin, entry_point, None);
            CompilationResult { result }
        } else {
            panic!("No Compiler can be created")
        }
    }
}

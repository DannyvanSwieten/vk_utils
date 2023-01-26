use byteorder::ReadBytesExt;
use spirv_reflect::{
    types::{
        ReflectBlockVariable, ReflectDescriptorBinding, ReflectDescriptorSet,
        ReflectInterfaceVariable,
    },
    ShaderModule,
};
use std::fs::File;
use std::path::Path;

use shaderc::{CompilationArtifact, Compiler, ShaderKind};

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
    module: ShaderModule,
}

impl ShaderReflection {
    pub fn entry_point(&self) -> String {
        self.module.get_entry_point_name()
    }

    pub fn outputs(&self) -> Option<Vec<ReflectInterfaceVariable>> {
        match self.module.enumerate_output_variables(None) {
            Ok(outputs) => Some(outputs),
            Err(_) => None,
        }
    }

    pub fn descriptor_sets(&self) -> Option<Vec<ReflectDescriptorSet>> {
        match self.module.enumerate_descriptor_sets(None) {
            Ok(sets) => Some(sets),
            Err(_) => None,
        }
    }

    pub fn bindings(&self) -> Option<Vec<ReflectDescriptorBinding>> {
        match self.module.enumerate_descriptor_bindings(None) {
            Ok(bindings) => Some(bindings),
            Err(_) => None,
        }
    }

    pub fn push_constants(&self) -> Option<Vec<ReflectBlockVariable>> {
        match self.module.enumerate_push_constant_blocks(None) {
            Ok(push_constant_blocks) => Some(push_constant_blocks),
            Err(_) => None,
        }
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
            Ok(s) => match ShaderModule::load_u32_data(s.as_binary()) {
                Ok(module) => ShaderReflection { module },
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
            let result = compiler.compile_into_spirv(src, kind, origin, entry_point, None);
            CompilationResult { result }
        } else {
            panic!("No Compiler can be created")
        }
    }
}

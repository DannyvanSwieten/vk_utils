use ash::vk::{ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags};
use ash::Device;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use byteorder::ReadBytesExt;
use std::fs::File;

use crate::device_context::DeviceContext;

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

pub struct ShaderLibraryEntry {
    module: ShaderModule,
    stage: ShaderStageFlags,
    entry_point: String,
}

impl ShaderLibraryEntry {
    pub fn module(&self) -> &ShaderModule {
        &self.module
    }
}
pub struct ShaderLibrary {
    device: Rc<DeviceContext>,
    entries: HashMap<String, ShaderLibraryEntry>,
    root: String,
}

impl ShaderLibrary {
    pub fn new(device: Rc<DeviceContext>, root: &Path) -> Self {
        Self {
            device,
            entries: HashMap::new(),
            root: root.to_str().unwrap().to_string(),
        }
    }

    pub fn add_spirv(
        &mut self,
        stage: ShaderStageFlags,
        id: &str,
        entry_point: &str,
        code: &[u32],
    ) {
        let info = ShaderModuleCreateInfo::builder().code(code).build();
        let module = unsafe {
            self.device
                .handle()
                .create_shader_module(&info, None)
                .expect("Shader Module creation failed")
        };

        self.entries.insert(
            String::from(id),
            ShaderLibraryEntry {
                module,
                entry_point: String::from(entry_point),
                stage,
            },
        );
    }
    pub fn add_spirv_from_file(
        &mut self,
        stage: ShaderStageFlags,
        id: &str,
        entry_point: &str,
        path: &Path,
    ) {
        if let Some(p) = Path::new(&self.root).join(path).to_str() {
            let spirv = load_spirv(p);
            self.add_spirv(stage, id, entry_point, &spirv);
        }
    }
    pub fn get(&self, id: &str) -> Option<&ShaderLibraryEntry> {
        self.entries.get(&String::from(id))
    }

    pub fn get_unchecked(&self, id: &str) -> &ShaderLibraryEntry {
        self.entries.get(&String::from(id)).unwrap()
    }
}

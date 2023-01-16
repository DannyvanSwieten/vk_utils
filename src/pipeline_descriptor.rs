use ash::vk::{self, DescriptorPoolSize, PushConstantRange, ShaderStageFlags};

pub struct PipelineDescriptor {
    uniforms: Vec<DescriptorPoolSize>,
    push_constant_range: Option<vk::PushConstantRange>,
}

impl PipelineDescriptor {
    pub fn new() -> Self {
        Self {
            uniforms: Vec::new(),
            push_constant_range: None,
        }
    }

    pub fn with_push_constants<T: Sized>(mut self, stage: ShaderStageFlags) {
        self.push_constant_range = Some(
            *PushConstantRange::builder()
                .size(std::mem::size_of::<T>() as _)
                .stage_flags(stage),
        )
    }

    pub fn with_uniform(mut self, uniform_type: ash::vk::DescriptorType, count: u32) -> Self {
        self.uniforms.push(DescriptorPoolSize {
            ty: uniform_type,
            descriptor_count: count,
        });
        self
    }

    pub fn with_uniforms(mut self, uniforms: &[(ash::vk::DescriptorType, u32)]) -> Self {
        self.uniforms
            .extend(uniforms.iter().map(|(t, c)| DescriptorPoolSize {
                ty: *t,
                descriptor_count: *c,
            }));
        self
    }
}

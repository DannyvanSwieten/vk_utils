use std::{collections::HashMap, ffi::CString, path::Path, rc::Rc};

use ash::vk::{
    ComputePipelineCreateInfo, DescriptorBufferInfo, DescriptorImageInfo, DescriptorPoolCreateInfo,
    DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout,
    DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType, Pipeline,
    PipelineCache, PipelineLayout, PipelineLayoutCreateInfo, PipelineShaderStageCreateInfo,
    PushConstantRange, ShaderModuleCreateInfo, ShaderStageFlags, WriteDescriptorSet,
};
use rspirv_reflect::BindingCount;
use shaderc::ShaderKind;

use crate::{
    buffer_resource::BufferResource,
    device_context::DeviceContext,
    image2d_resource::Image2DResource,
    image_resource::ImageResource,
    shader_compiler::{ShaderCompiler, ShaderReflection},
};

pub struct ComputePipeline {
    device: Rc<DeviceContext>,
    pipeline_layout: PipelineLayout,
    pipeline: Pipeline,
    descriptor_sets: Vec<DescriptorSet>,
}

impl ComputePipeline {
    pub fn handle(&self) -> &Pipeline {
        &self.pipeline
    }

    pub fn layout(&self) -> &PipelineLayout {
        &self.pipeline_layout
    }

    pub fn descriptor_sets(&self) -> &[DescriptorSet] {
        &self.descriptor_sets
    }

    pub fn set_storage_buffer(&mut self, set: usize, binding: usize, buffer: &BufferResource) {
        let buffer_info = [DescriptorBufferInfo::default()
            .buffer(buffer.buffer)
            .range(buffer.content_size())];
        let write = WriteDescriptorSet::default()
            .buffer_info(&buffer_info)
            .descriptor_type(DescriptorType::STORAGE_BUFFER)
            .dst_set(self.descriptor_sets[set])
            .dst_binding(binding as _);
        unsafe { self.device.handle().update_descriptor_sets(&[write], &[]) }
    }

    pub fn set_storage_image(&mut self, set: usize, binding: usize, image: &Image2DResource) {
        let image_info = [DescriptorImageInfo::default()
            .image_view(image.view())
            .image_layout(image.layout())];
        let write = WriteDescriptorSet::default()
            .image_info(&image_info)
            .descriptor_type(DescriptorType::STORAGE_IMAGE)
            .dst_set(self.descriptor_sets[set])
            .dst_binding(binding as _);
        unsafe { self.device.handle().update_descriptor_sets(&[write], &[]) }
    }

    pub fn set_uniform_buffer(&mut self, set: usize, binding: usize, buffer: &BufferResource) {
        let buffer_info = [DescriptorBufferInfo::default()
            .buffer(buffer.buffer)
            .range(buffer.size())];
        let write = WriteDescriptorSet::default()
            .buffer_info(&buffer_info)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .dst_set(self.descriptor_sets[set])
            .dst_binding(binding as _);
        unsafe { self.device.handle().update_descriptor_sets(&[write], &[]) }
    }

    fn create_descriptor_set_bindings(
        reflection: &ShaderReflection,
    ) -> HashMap<u32, Vec<DescriptorSetLayoutBinding>> {
        let mut sets = HashMap::<u32, Vec<DescriptorSetLayoutBinding>>::new();
        if let Some(descriptor_sets) = reflection.descriptor_sets() {
            #[cfg(debug_assertions)]
            {
                println!("Reflection: {:?}", descriptor_sets);
            }
            for (set, descriptors) in descriptor_sets {
                for (index, descriptor) in descriptors {
                    // let mut v = Vec::new();
                    let mut b = DescriptorSetLayoutBinding::default()
                        .binding(index)
                        .stage_flags(ShaderStageFlags::COMPUTE);
                    match descriptor.binding_count {
                        BindingCount::One => {
                            b = b.descriptor_count(1);
                        }
                        BindingCount::StaticSized(size) => {
                            b = b.descriptor_count(size as _);
                        }
                        BindingCount::Unbounded => {
                            b = b.descriptor_count(0);
                        }
                    }

                    match descriptor.ty {
                        rspirv_reflect::DescriptorType::SAMPLER => {
                            b = b.descriptor_type(DescriptorType::SAMPLER);
                        }
                        rspirv_reflect::DescriptorType::COMBINED_IMAGE_SAMPLER => {
                            b = b.descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER);
                        }
                        rspirv_reflect::DescriptorType::SAMPLED_IMAGE => {
                            b = b.descriptor_type(DescriptorType::SAMPLED_IMAGE);
                        }
                        rspirv_reflect::DescriptorType::STORAGE_IMAGE => {
                            b = b.descriptor_type(DescriptorType::STORAGE_IMAGE);
                        }
                        rspirv_reflect::DescriptorType::UNIFORM_TEXEL_BUFFER => {
                            b = b.descriptor_type(DescriptorType::UNIFORM_TEXEL_BUFFER);
                        }
                        rspirv_reflect::DescriptorType::STORAGE_TEXEL_BUFFER => {
                            b = b.descriptor_type(DescriptorType::STORAGE_TEXEL_BUFFER);
                        }
                        rspirv_reflect::DescriptorType::UNIFORM_BUFFER => {
                            b = b.descriptor_type(DescriptorType::UNIFORM_BUFFER);
                        }
                        rspirv_reflect::DescriptorType::STORAGE_BUFFER => {
                            b = b.descriptor_type(DescriptorType::STORAGE_BUFFER);
                        }
                        rspirv_reflect::DescriptorType::UNIFORM_BUFFER_DYNAMIC => {
                            b = b.descriptor_type(DescriptorType::UNIFORM_BUFFER_DYNAMIC);
                        }
                        rspirv_reflect::DescriptorType::STORAGE_BUFFER_DYNAMIC => {
                            b = b.descriptor_type(DescriptorType::STORAGE_BUFFER_DYNAMIC);
                        }
                        rspirv_reflect::DescriptorType::INPUT_ATTACHMENT => {
                            b = b.descriptor_type(DescriptorType::INPUT_ATTACHMENT);
                        }
                        rspirv_reflect::DescriptorType::ACCELERATION_STRUCTURE_NV => {
                            b = b.descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_NV);
                        }
                        rspirv_reflect::DescriptorType::ACCELERATION_STRUCTURE_KHR => {
                            b = b.descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_KHR);
                        }
                        rspirv_reflect::DescriptorType::INLINE_UNIFORM_BLOCK_EXT => {
                            b = b.descriptor_type(DescriptorType::INLINE_UNIFORM_BLOCK_EXT);
                        }
                        _ => {} // todo!(),
                    }

                    sets.entry(set).or_default().push(b);
                }
            }
        }
        sets
    }

    pub fn new_from_source_file(
        path: &Path,
        device: Rc<DeviceContext>,
        max_frames_in_flight: u32,
        entry_point: &str,
        explicit_bindings: Option<HashMap<u32, Vec<DescriptorSetLayoutBinding>>>,
    ) -> Option<Self> {
        let src = std::fs::read_to_string(path);
        match src {
            Ok(src) => Self::new_from_source_string(
                device,
                max_frames_in_flight,
                &src,
                entry_point,
                explicit_bindings,
            ),
            Err(_) => None,
        }
    }

    pub fn new_from_source_string(
        device: Rc<DeviceContext>,
        max_frames_in_flight: u32,
        src: &str,
        entry_point: &str,
        explicit_bindings: Option<HashMap<u32, Vec<DescriptorSetLayoutBinding>>>,
    ) -> Option<Self> {
        let result = ShaderCompiler::compile_string(src, ShaderKind::Compute, "", entry_point);
        let this = if !result.failed() {
            let reflection = result.reflect();
            let mut descriptor_set_bindings = Self::create_descriptor_set_bindings(&reflection);
            if let Some(explicit_bindings) = explicit_bindings {
                for (index, bindings) in explicit_bindings {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        descriptor_set_bindings.entry(index)
                    {
                        e.insert(bindings);
                    } else {
                        descriptor_set_bindings
                            .get_mut(&index)
                            .unwrap()
                            .extend(bindings.iter())
                    }
                }
            }

            let mut constant_ranges = Vec::new();
            if let Ok(push_blocks) = reflection.push_constant_ranges() {
                push_blocks.into_iter().for_each(|block| {
                    #[cfg(debug_assertions)]
                    {
                        println!("Push constant range: {:?}", block);
                    }
                    constant_ranges.push(
                        PushConstantRange::default()
                            .size(block.size)
                            .offset(block.offset)
                            .stage_flags(ShaderStageFlags::COMPUTE),
                    );
                });
            }

            let mut layouts = vec![DescriptorSetLayout::default(); descriptor_set_bindings.len()];
            let mut pool_sizes = Vec::new();
            for (index, set) in &descriptor_set_bindings {
                let mut builder = DescriptorSetLayoutCreateInfo::default();
                builder = builder.bindings(set);
                let layout = unsafe {
                    device
                        .handle()
                        .create_descriptor_set_layout(&builder, None)
                        .expect("Creating descriptorset layout failed: {}")
                };

                layouts[*index as usize] = layout;

                for binding in set {
                    let size = DescriptorPoolSize::default()
                        .ty(binding.descriptor_type)
                        .descriptor_count(binding.descriptor_count);
                    pool_sizes.push(size);
                }
            }

            let pipeline_info_builder = PipelineLayoutCreateInfo::default()
                .set_layouts(&layouts)
                .push_constant_ranges(&constant_ranges);
            let pipeline_layout = unsafe {
                device
                    .handle()
                    .create_pipeline_layout(&pipeline_info_builder, None)
                    .expect("Pipeline layout creation failed")
            };

            let shader_info = ShaderModuleCreateInfo::default().code(result.spirv());
            let shader_module = unsafe {
                device
                    .handle()
                    .create_shader_module(&shader_info, None)
                    .expect("Shader module creation failed")
            };

            let s = CString::new(entry_point).expect("String creation failed");
            let shader_stage_info = PipelineShaderStageCreateInfo::default()
                .module(shader_module)
                .stage(ShaderStageFlags::COMPUTE)
                .name(&s);

            let compute_pipeline_info = ComputePipelineCreateInfo::default()
                .layout(pipeline_layout)
                .stage(shader_stage_info);

            let pipeline = unsafe {
                device
                    .handle()
                    .create_compute_pipelines(PipelineCache::null(), &[compute_pipeline_info], None)
                    .expect("Pipeline creation failed")[0]
            };

            let pool_info = DescriptorPoolCreateInfo::default()
                .pool_sizes(&pool_sizes)
                .max_sets(max_frames_in_flight * descriptor_set_bindings.len() as u32);

            let pool = unsafe {
                device
                    .handle()
                    .create_descriptor_pool(&pool_info, None)
                    .expect("Descriptor pool creation failed")
            };
            let allocation_info = DescriptorSetAllocateInfo::default()
                .descriptor_pool(pool)
                .set_layouts(&layouts);
            let descriptor_sets = unsafe {
                device
                    .handle()
                    .allocate_descriptor_sets(&allocation_info)
                    .expect("Descriptor set allocation failed")
            };

            Some(Self {
                device,
                pipeline_layout,
                pipeline,
                descriptor_sets,
            })
        } else {
            println!("{}", result.error_string());
            None
        };

        this
    }
}

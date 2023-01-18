use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    path::Path,
    rc::Rc,
};

use ash::vk::{
    ComputePipelineCreateInfo, DescriptorBufferInfo, DescriptorImageInfo, DescriptorPoolCreateInfo,
    DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout,
    DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType, Pipeline,
    PipelineCache, PipelineLayout, PipelineLayoutCreateInfo, PipelineShaderStageCreateInfo,
    PushConstantRange, ShaderModuleCreateInfo, ShaderStageFlags, WriteDescriptorSet,
};
use shaderc::ShaderKind;
use spirv_reflect::types::ReflectDescriptorType;

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
        let buffer_info = [*DescriptorBufferInfo::builder()
            .buffer(buffer.buffer)
            .range(buffer.content_size())];
        let write = WriteDescriptorSet::builder()
            .buffer_info(&buffer_info)
            .descriptor_type(DescriptorType::STORAGE_BUFFER)
            .dst_set(self.descriptor_sets[set])
            .dst_binding(binding as _);
        unsafe { self.device.handle().update_descriptor_sets(&[*write], &[]) }
    }

    pub fn set_storage_image(&mut self, set: usize, binding: usize, image: &Image2DResource) {
        let image_info = [*DescriptorImageInfo::builder()
            .image_view(image.view())
            .image_layout(image.layout())];
        let write = WriteDescriptorSet::builder()
            .image_info(&image_info)
            .descriptor_type(DescriptorType::STORAGE_IMAGE)
            .dst_set(self.descriptor_sets[set])
            .dst_binding(binding as _);
        unsafe { self.device.handle().update_descriptor_sets(&[*write], &[]) }
    }

    pub fn set_uniform_buffer(&mut self, set: usize, binding: usize, buffer: &BufferResource) {
        let buffer_info = [*DescriptorBufferInfo::builder()
            .buffer(buffer.buffer)
            .range(buffer.size())];
        let write = WriteDescriptorSet::builder()
            .buffer_info(&buffer_info)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .dst_set(self.descriptor_sets[set])
            .dst_binding(binding as _);
        unsafe { self.device.handle().update_descriptor_sets(&[*write], &[]) }
    }

    fn create_descriptor_set_bindings(
        reflection: &ShaderReflection,
    ) -> HashMap<u32, Vec<DescriptorSetLayoutBinding>> {
        let mut sets = HashMap::<u32, Vec<DescriptorSetLayoutBinding>>::new();
        if let Some(bindings) = reflection.bindings() {
            for binding in bindings {
                let mut b = DescriptorSetLayoutBinding::builder();
                b = b
                    .binding(binding.binding)
                    .descriptor_count(binding.count)
                    .stage_flags(ShaderStageFlags::COMPUTE);
                match binding.descriptor_type {
                    ReflectDescriptorType::Undefined => todo!(),
                    ReflectDescriptorType::Sampler => {
                        b = b.descriptor_type(DescriptorType::SAMPLER);
                    }
                    ReflectDescriptorType::CombinedImageSampler => {
                        b = b.descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER);
                    }
                    ReflectDescriptorType::SampledImage => {
                        b = b.descriptor_type(DescriptorType::SAMPLED_IMAGE);
                    }
                    ReflectDescriptorType::StorageImage => {
                        b = b.descriptor_type(DescriptorType::STORAGE_IMAGE);
                    }
                    ReflectDescriptorType::UniformTexelBuffer => {
                        b = b.descriptor_type(DescriptorType::UNIFORM_TEXEL_BUFFER);
                    }
                    ReflectDescriptorType::StorageTexelBuffer => {
                        b = b.descriptor_type(DescriptorType::STORAGE_TEXEL_BUFFER);
                    }
                    ReflectDescriptorType::UniformBuffer => {
                        b = b.descriptor_type(DescriptorType::UNIFORM_BUFFER);
                    }
                    ReflectDescriptorType::StorageBuffer => {
                        b = b.descriptor_type(DescriptorType::STORAGE_BUFFER);
                    }
                    ReflectDescriptorType::UniformBufferDynamic => {
                        b = b.descriptor_type(DescriptorType::UNIFORM_BUFFER_DYNAMIC);
                    }
                    ReflectDescriptorType::StorageBufferDynamic => {
                        b = b.descriptor_type(DescriptorType::STORAGE_BUFFER_DYNAMIC);
                    }
                    ReflectDescriptorType::InputAttachment => {
                        b = b.descriptor_type(DescriptorType::INPUT_ATTACHMENT);
                    }
                    ReflectDescriptorType::AccelerationStructureNV => {
                        b = b.descriptor_type(DescriptorType::ACCELERATION_STRUCTURE_NV);
                    }
                }
                sets.entry(binding.set).or_insert_with(Vec::new).push(*b);
            }
        }
        sets
    }

    pub fn new_from_source_file(
        path: &Path,
        device: Rc<DeviceContext>,
        max_frames_in_flight: u32,
        entry_point: &str,
    ) -> Option<Self> {
        let src = std::fs::read_to_string(path);
        match src {
            Ok(src) => {
                Self::new_from_source_string(device, max_frames_in_flight, &src, entry_point)
            }
            Err(_) => None,
        }
    }

    pub fn new_from_source_string(
        device: Rc<DeviceContext>,
        max_frames_in_flight: u32,
        src: &str,
        entry_point: &str,
    ) -> Option<Self> {
        let result = ShaderCompiler::compile_string(src, ShaderKind::Compute, "", entry_point);
        let this = if !result.failed() {
            let reflection = result.reflect();
            let descriptor_set_bindings = Self::create_descriptor_set_bindings(&reflection);
            let mut layouts = vec![DescriptorSetLayout::default(); descriptor_set_bindings.len()];
            let mut pool_sizes = Vec::new();
            for (index, set) in descriptor_set_bindings {
                let mut builder = DescriptorSetLayoutCreateInfo::builder();
                builder = builder.bindings(&set);
                let layout = unsafe {
                    device
                        .handle()
                        .create_descriptor_set_layout(&builder, None)
                        .expect("Creating descriptorset layout failed: {}")
                };

                layouts[index as usize] = layout;

                for binding in set {
                    let size = DescriptorPoolSize::builder()
                        .ty(binding.descriptor_type)
                        .descriptor_count(binding.descriptor_count);
                    pool_sizes.push(*size);
                }
            }

            let pipeline_info_builder = PipelineLayoutCreateInfo::builder().set_layouts(&layouts);
            let pipeline_layout = unsafe {
                device
                    .handle()
                    .create_pipeline_layout(&pipeline_info_builder, None)
                    .expect("Pipeline layout creation failed")
            };

            let shader_info = ShaderModuleCreateInfo::builder().code(result.spirv());
            let shader_module = unsafe {
                device
                    .handle()
                    .create_shader_module(&shader_info, None)
                    .expect("Shader module creation failed")
            };

            let s = CString::new(entry_point).expect("String creation failed");
            let shader_stage_info = PipelineShaderStageCreateInfo::builder()
                .module(shader_module)
                .stage(ShaderStageFlags::COMPUTE)
                .name(&s);

            let compute_pipeline_info = ComputePipelineCreateInfo::builder()
                .layout(pipeline_layout)
                .stage(*shader_stage_info);

            let pipeline = unsafe {
                device
                    .handle()
                    .create_compute_pipelines(
                        PipelineCache::null(),
                        &[*compute_pipeline_info],
                        None,
                    )
                    .expect("Pipeline creation failed")[0]
            };

            let pool_info = DescriptorPoolCreateInfo::builder()
                .pool_sizes(&pool_sizes)
                .max_sets(max_frames_in_flight);

            let pool = unsafe {
                device
                    .handle()
                    .create_descriptor_pool(&pool_info, None)
                    .expect("Descriptor pool creation failed")
            };
            let allocation_info = DescriptorSetAllocateInfo::builder()
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

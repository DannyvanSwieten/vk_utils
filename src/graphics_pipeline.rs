use std::{
    ffi::{CStr, CString},
    rc::Rc,
};

use ash::vk::{
    DynamicState, GraphicsPipelineCreateInfo, Pipeline, PipelineCache,
    PipelineColorBlendStateCreateInfo, PipelineDepthStencilStateCreateInfo,
    PipelineDynamicStateCreateInfo, PipelineLayout, PipelineMultisampleStateCreateInfo,
    PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo,
    PipelineTessellationStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode, RenderPass,
    ShaderModule, ShaderStageFlags, Viewport,
};

use crate::device_context::DeviceContext;

#[derive(Default, Clone)]
pub struct GraphicsPipelineState {
    blend_state: Option<PipelineColorBlendStateCreateInfo>,
    depth_stencil_state: Option<PipelineDepthStencilStateCreateInfo>,
    dynamic_state: Option<PipelineDynamicStateCreateInfo>,
    multisample_state: Option<PipelineMultisampleStateCreateInfo>,
    rasterization_state: Option<PipelineRasterizationStateCreateInfo>,
    shader_stage_state: Vec<PipelineShaderStageCreateInfo>,
    tesselation_state: Option<PipelineTessellationStateCreateInfo>,
    viewport_state: Option<PipelineViewportStateCreateInfo>,
    viewports: Vec<Viewport>,
}

impl GraphicsPipelineState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_viewport(mut self, width: u32, height: u32) -> Self {
        let vp = *Viewport::builder()
            .width(width as f32)
            .height(height as f32);
        self.viewports = [vp].to_vec();
        let s = *PipelineViewportStateCreateInfo::builder().viewports(&self.viewports);
        if self.viewport_state.is_none() {
            self.viewport_state = Some(s)
        }

        self
    }

    pub fn with_polygon_mode(mut self, mode: PolygonMode) -> Self {
        if self.rasterization_state.is_none() {
            self.rasterization_state = Some(PipelineRasterizationStateCreateInfo::default())
        }

        self.rasterization_state.unwrap().polygon_mode = mode;
        self
    }

    pub fn with_depth_testing(mut self) -> Self {
        if self.depth_stencil_state.is_none() {
            self.depth_stencil_state = Some(PipelineDepthStencilStateCreateInfo::default())
        }

        self.depth_stencil_state.unwrap().depth_test_enable = 1;
        self
    }

    pub fn with_depth_writing(mut self) -> Self {
        if self.depth_stencil_state.is_none() {
            self.depth_stencil_state = Some(PipelineDepthStencilStateCreateInfo::default())
        }
        self.depth_stencil_state.unwrap().depth_write_enable = 1;
        self
    }

    pub fn with_vertex_shader(mut self, name: &str, module: &ShaderModule) -> Self {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::VERTEX)
                .name(&CString::new(name).expect("Name unwrap failed"))
                .build(),
        );

        self
    }

    pub fn with_fragment_shader(mut self, name: &str, module: &ShaderModule) -> Self {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::FRAGMENT)
                .name(&CString::new(name).expect("Name unwrap failed"))
                .build(),
        );

        self
    }

    pub fn with_geometry_shader(mut self, module: &ShaderModule) -> Self {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::GEOMETRY)
                .build(),
        );

        self
    }

    pub fn with_tesselation_control_shader(mut self, module: &ShaderModule) -> Self {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::TESSELLATION_CONTROL)
                .build(),
        );

        self
    }

    pub fn with_tesselation_evaluation_shader(mut self, module: &ShaderModule) -> Self {
        self.shader_stage_state.push(
            PipelineShaderStageCreateInfo::builder()
                .module(*module)
                .stage(ShaderStageFlags::TESSELLATION_EVALUATION)
                .build(),
        );

        self
    }
}

pub struct GraphicsPipeline {
    device: Rc<DeviceContext>,
    pipeline_layout: PipelineLayout,
    pipeline: Pipeline,
}

impl GraphicsPipeline {
    pub fn new(device: Rc<DeviceContext>, state: &GraphicsPipelineState) -> Self {
        let dynamic_state = state.dynamic_state.unwrap_or_default();
        let rasterizer_state = state.rasterization_state.unwrap_or_default();
        let blend_state = state.blend_state.unwrap_or_default();

        let info = GraphicsPipelineCreateInfo::builder()
            .dynamic_state(&dynamic_state)
            .rasterization_state(&rasterizer_state)
            .color_blend_state(&blend_state)
            .stages(&state.shader_stage_state);

        let pipelines = unsafe {
            device
                .handle()
                .create_graphics_pipelines(PipelineCache::null(), &[*info], None)
                .expect("Pipeline Creation Failed")
        };
        Self {
            device,
            pipeline_layout: PipelineLayout::null(),
            pipeline: pipelines[0],
        }
    }

    pub fn handle(&self) -> &Pipeline {
        &self.pipeline
    }

    pub fn layout(&self) -> &PipelineLayout {
        &self.pipeline_layout
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe { self.device.handle().destroy_pipeline(self.pipeline, None) }
    }
}

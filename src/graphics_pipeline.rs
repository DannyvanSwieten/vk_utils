use std::{ffi::CString, rc::Rc};

use ash::vk::{
    Bool32, CullModeFlags, FrontFace, GraphicsPipelineCreateInfo, Pipeline, PipelineCache,
    PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo, PipelineLayout,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineShaderStageCreateInfo, PipelineTessellationStateCreateInfo,
    PipelineViewportStateCreateInfo, PolygonMode, ShaderModule, ShaderStageFlags, Viewport,
};

use crate::device_context::DeviceContext;

#[derive(Clone)]
pub struct DepthState {
    pub depth_test_enable: u32,
    pub depth_write_enable: u32,
    pub depth_compare_op: u32,
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            depth_test_enable: 0,
            depth_write_enable: 0,
            depth_compare_op: 0,
        }
    }
}

#[derive(Clone)]
pub struct MultiSampleState {
    pub sample_shading_enable: u32,
    pub rasterization_samples: u32,
}

#[derive(Clone)]
pub struct RasterizerState {
    pub rasterizer_discard_enable: Bool32,
    pub polygon_mode: PolygonMode,
    pub cull_mode: CullModeFlags,
    pub front_face: FrontFace,
}

impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            rasterizer_discard_enable: 0,
            polygon_mode: PolygonMode::FILL,
            cull_mode: CullModeFlags::BACK,
            front_face: FrontFace::COUNTER_CLOCKWISE,
        }
    }
}

#[derive(Default, Clone)]
pub struct GraphicsPipelineState {
    blend_state: Option<PipelineColorBlendAttachmentState>,
    depth_stencil_state: Option<DepthState>,
    multisample_state: Option<MultiSampleState>,
    rasterization_state: Option<RasterizerState>,
    viewports: Vec<Viewport>,
}

impl GraphicsPipelineState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_viewport(mut self, width: u32, height: u32) -> Self {
        let vp = Viewport::default()
            .width(width as f32)
            .height(height as f32);
        self.viewports = [vp].to_vec();
        self
    }

    pub fn with_polygon_mode(mut self, mode: PolygonMode) -> Self {
        if self.rasterization_state.is_none() {
            self.rasterization_state = Some(RasterizerState::default())
        }

        self.rasterization_state.as_mut().unwrap().polygon_mode = mode;
        self
    }

    pub fn with_depth_testing(mut self) -> Self {
        if self.depth_stencil_state.is_none() {
            self.depth_stencil_state = Some(DepthState::default())
        }

        self.depth_stencil_state.as_mut().unwrap().depth_test_enable = 1;
        self
    }

    pub fn with_depth_writing(mut self) -> Self {
        if self.depth_stencil_state.is_none() {
            self.depth_stencil_state = Some(DepthState::default())
        }
        self.depth_stencil_state
            .as_mut()
            .unwrap()
            .depth_write_enable = 1;
        self
    }

    // pub fn with_vertex_shader(mut self, name: &str, module: &ShaderModule) -> Self {
    //     self.shader_stage_state.push(
    //         PipelineShaderStageCreateInfo::default()
    //             .module(*module)
    //             .stage(ShaderStageFlags::VERTEX)
    //             .name(&CString::new(name).expect("Name unwrap failed")),
    //     );

    //     self
    // }

    // pub fn with_fragment_shader(mut self, name: &str, module: &ShaderModule) -> Self {
    //     self.shader_stage_state.push(
    //         PipelineShaderStageCreateInfo::default()
    //             .module(*module)
    //             .stage(ShaderStageFlags::FRAGMENT)
    //             .name(&CString::new(name).expect("Name unwrap failed")),
    //     );

    //     self
    // }

    // pub fn with_geometry_shader(mut self, module: &ShaderModule) -> Self {
    //     self.shader_stage_state.push(
    //         PipelineShaderStageCreateInfo::default()
    //             .module(*module)
    //             .stage(ShaderStageFlags::GEOMETRY),
    //     );

    //     self
    // }

    // pub fn with_tesselation_control_shader(mut self, module: &ShaderModule) -> Self {
    //     self.shader_stage_state.push(
    //         PipelineShaderStageCreateInfo::default()
    //             .module(*module)
    //             .stage(ShaderStageFlags::TESSELLATION_CONTROL),
    //     );

    //     self
    // }

    // pub fn with_tesselation_evaluation_shader(mut self, module: &ShaderModule) -> Self {
    //     self.shader_stage_state.push(
    //         PipelineShaderStageCreateInfo::default()
    //             .module(*module)
    //             .stage(ShaderStageFlags::TESSELLATION_EVALUATION),
    //     );

    //     self
    // }
}

pub struct GraphicsPipeline {
    device: Rc<DeviceContext>,
    pipeline_layout: PipelineLayout,
    pipeline: Pipeline,
}

impl GraphicsPipeline {
    pub fn new(device: Rc<DeviceContext>, state: &GraphicsPipelineState) -> Self {
        // let dynamic_state = state.dynamic_state.unwrap_or_default();
        // let rasterizer_state = state.rasterization_state.unwrap_or_default();
        // let blend_state = state.blend_state.unwrap_or_default();

        let info = GraphicsPipelineCreateInfo::default();

        let pipelines = unsafe {
            device
                .handle()
                .create_graphics_pipelines(PipelineCache::null(), &[info], None)
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

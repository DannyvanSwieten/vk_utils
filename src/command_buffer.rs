use std::rc::Rc;

use ash::vk::{
    AccessFlags, Buffer, BufferImageCopy, BufferMemoryBarrier, ClearColorValue, ClearValue,
    CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferUsageFlags, DependencyFlags,
    DescriptorSet, Extent2D, Extent3D, FenceCreateInfo, Filter, Framebuffer, ImageAspectFlags,
    ImageBlit, ImageLayout, ImageMemoryBarrier, ImageSubresourceLayers, ImageSubresourceRange,
    Offset3D, PipelineBindPoint, PipelineLayout, PipelineStageFlags, Rect2D, RenderPassBeginInfo,
    ShaderStageFlags, SubmitInfo, SubpassContents,
};

use crate::buffer_resource::BufferResource;
use crate::device_context::DeviceContext;
use crate::image_resource::ImageResource;
use crate::pipeline_descriptor::ComputePipeline;
use crate::queue::CommandQueue;
use crate::wait_handle::WaitHandle;

pub struct CommandBuffer {
    device: Rc<DeviceContext>,
    queue: Rc<CommandQueue>,
    handle: Vec<ash::vk::CommandBuffer>,
}

impl CommandBuffer {
    pub fn new(queue: Rc<CommandQueue>) -> Self {
        let device = queue.device();
        let info = CommandBufferAllocateInfo::default()
            .command_buffer_count(1)
            .command_pool(queue.pool());
        let handle = unsafe { device.handle().allocate_command_buffers(&info) };
        Self {
            device,
            queue,
            handle: handle.expect("Command buffer allocation failed"),
        }
    }

    pub(crate) fn queue(&self) -> Rc<CommandQueue> {
        self.queue.clone()
    }

    pub fn begin(&mut self) {
        let begin_info =
            CommandBufferBeginInfo::default().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            let success = self
                .device
                .handle()
                .begin_command_buffer(self.handle(), &begin_info);

            match success {
                Ok(_) => (),
                Err(_) => panic!(),
            }
        }
    }

    pub fn record_handle<F>(&mut self, f: F)
    where
        F: FnOnce(ash::vk::CommandBuffer) -> ash::vk::CommandBuffer,
    {
        self.handle[0] = f(self.handle());
    }

    pub fn end(&mut self) {
        unsafe {
            let success = self.device.handle().end_command_buffer(self.handle());

            match success {
                Ok(_) => (),
                Err(_) => panic!(),
            }
        }
    }

    pub fn submit(self) -> WaitHandle {
        unsafe {
            let info = FenceCreateInfo::default();
            let fence = self
                .device
                .handle()
                .create_fence(&info, None)
                .expect("Fence creation failed");

            self.device
                .handle()
                .end_command_buffer(self.handle())
                .expect("Command Buffer end failed");
            let submit_info = SubmitInfo::default().command_buffers(&self.handle);
            self.device
                .handle()
                .queue_submit(self.queue.handle(), &[submit_info], fence)
                .expect("Queue submit failed");

            WaitHandle::new(self, fence)
        }
    }

    pub fn bind_pipeline(&mut self, bind_point: PipelineBindPoint, pipeline: &ash::vk::Pipeline) {
        unsafe {
            self.device
                .handle()
                .cmd_bind_pipeline(self.handle(), bind_point, *pipeline);
        }
    }

    pub fn bind_compute_pipeline(&mut self, pipeline: &ComputePipeline) {
        unsafe {
            self.device.handle().cmd_bind_pipeline(
                self.handle(),
                PipelineBindPoint::COMPUTE,
                *pipeline.handle(),
            );

            self.device.handle().cmd_bind_descriptor_sets(
                self.handle(),
                PipelineBindPoint::COMPUTE,
                *pipeline.layout(),
                0,
                pipeline.descriptor_sets(),
                &[],
            )
        }
    }

    pub fn dispatch_compute(&mut self, width: u32, height: u32, depth: u32) {
        unsafe {
            self.device
                .handle()
                .cmd_dispatch(self.handle(), width, height, depth)
        }
    }

    pub fn bind_descriptor_sets(
        &mut self,
        layout: &PipelineLayout,
        bind_point: PipelineBindPoint,
        sets: &[DescriptorSet],
    ) {
        unsafe {
            self.device.handle().cmd_bind_descriptor_sets(
                self.handle(),
                bind_point,
                *layout,
                0,
                sets,
                &[],
            )
        }
    }

    pub fn bind_vertex_buffer(&mut self, first_binding: u32, buffers: &[Buffer]) {
        unsafe {
            self.device
                .handle()
                .cmd_bind_vertex_buffers(self.handle(), first_binding, buffers, &[])
        }
    }

    pub fn draw_vertices(
        &mut self,
        vertex_count: u32,
        first_vertex: u32,
        instance_count: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.handle().cmd_draw(
                self.handle(),
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            )
        }
    }

    pub(crate) fn handle(&self) -> ash::vk::CommandBuffer {
        self.handle[0]
    }

    pub(crate) fn device(&self) -> Rc<DeviceContext> {
        self.device.clone()
    }

    pub fn buffer_resource_barrier(
        &mut self,
        buffer: &BufferResource,
        producer: PipelineStageFlags,
        consumer: PipelineStageFlags,
        source: AccessFlags,
        destination: AccessFlags,
    ) {
        let barrier = BufferMemoryBarrier::default()
            .buffer(buffer.buffer)
            .size(buffer.size())
            .src_access_mask(source)
            .dst_access_mask(destination);

        unsafe {
            self.device.handle().cmd_pipeline_barrier(
                self.handle(),
                producer,
                consumer,
                DependencyFlags::BY_REGION,
                &[],
                &[barrier],
                &[],
            );
        }
    }

    pub fn image_resource_transition(
        &mut self,
        image: &mut impl ImageResource,
        layout: ImageLayout,
    ) {
        let barrier = ImageMemoryBarrier::default()
            .old_layout(image.layout())
            .new_layout(layout)
            .image(image.handle())
            .src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(
                ImageSubresourceRange::default()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1),
            );

        unsafe {
            self.device.handle().cmd_pipeline_barrier(
                self.handle(),
                PipelineStageFlags::ALL_COMMANDS,
                PipelineStageFlags::ALL_COMMANDS,
                DependencyFlags::BY_REGION,
                &[],
                &[],
                &[barrier],
            );
        }

        image.set_layout(layout);
    }

    pub fn blit(&mut self, src: &impl ImageResource, dst: &mut impl ImageResource) {
        let regions = [ImageBlit::default()
            .dst_subresource(
                ImageSubresourceLayers::default()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1),
            )
            .dst_offsets([Offset3D::default(), Offset3D::default().z(1)])
            .src_offsets([Offset3D::default(), Offset3D::default().z(1)])
            .src_subresource(
                ImageSubresourceLayers::default()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1),
            )];
        unsafe {
            self.device.handle().cmd_blit_image(
                self.handle(),
                src.handle(),
                src.layout(),
                dst.handle(),
                dst.layout(),
                &regions,
                Filter::LINEAR,
            )
        }
    }

    pub fn color_image_transition(
        &mut self,
        image: &ash::vk::Image,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
    ) {
        let barrier = ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .image(*image)
            .src_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(ash::vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(
                ImageSubresourceRange::default()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1),
            );

        unsafe {
            self.device.handle().cmd_pipeline_barrier(
                self.handle(),
                PipelineStageFlags::ALL_COMMANDS,
                PipelineStageFlags::ALL_COMMANDS,
                DependencyFlags::BY_REGION,
                &[],
                &[],
                &[barrier],
            );
        }
    }

    pub fn clear_image(&mut self, image: &mut impl ImageResource, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            let value = ClearColorValue {
                float32: [r, g, b, a],
            };
            let range = [ImageSubresourceRange::default()
                .layer_count(1)
                .level_count(1)
                .aspect_mask(ImageAspectFlags::COLOR)];
            self.device.handle().cmd_clear_color_image(
                self.handle(),
                image.handle(),
                ImageLayout::GENERAL,
                &value,
                &range,
            )
        }
    }

    pub fn copy_image_to_buffer(
        &mut self,
        image: &impl ImageResource,
        buffer: &mut BufferResource,
    ) {
        let layer_info = ImageSubresourceLayers::default()
            .layer_count(1)
            .aspect_mask(ImageAspectFlags::COLOR);
        let copy = [BufferImageCopy::default()
            .image_extent(
                Extent3D::default()
                    .width(image.width())
                    .height(image.height())
                    .depth(image.depth()),
            )
            .image_subresource(layer_info)];

        unsafe {
            self.device.handle().cmd_copy_image_to_buffer(
                self.handle(),
                image.handle(),
                image.layout(),
                buffer.buffer,
                &copy,
            )
        }
    }

    pub fn copy_buffer_to_image(
        &mut self,
        buffer: &BufferResource,
        image: &mut impl ImageResource,
    ) {
        let layer_info = ImageSubresourceLayers::default()
            .layer_count(1)
            .aspect_mask(ImageAspectFlags::COLOR);
        let copy = [BufferImageCopy::default()
            .image_extent(
                Extent3D::default()
                    .width(image.width())
                    .height(image.height())
                    .depth(image.depth()),
            )
            .image_subresource(layer_info)];

        unsafe {
            self.device.handle().cmd_copy_buffer_to_image(
                self.handle(),
                buffer.buffer,
                image.handle(),
                image.layout(),
                &copy,
            )
        }
    }

    pub fn begin_render_pass(
        &mut self,
        render_pass: &crate::renderpass::RenderPass,
        framebuffer: &Framebuffer,
        width: u32,
        height: u32,
    ) {
        let info = RenderPassBeginInfo::default()
            .render_pass(*render_pass.handle())
            .clear_values(&[ClearValue {
                color: ClearColorValue {
                    float32: [0.0, 1.0, 0.0, 1.0],
                },
            }])
            .render_area(Rect2D::default().extent(Extent2D::default().width(width).height(height)))
            .framebuffer(*framebuffer);

        unsafe {
            self.device.handle().cmd_begin_render_pass(
                self.handle(),
                &info,
                SubpassContents::INLINE,
            )
        }
    }

    pub fn end_render_pass(&mut self) {
        unsafe { self.device.handle().cmd_end_render_pass(self.handle()) }
    }

    pub fn push_compute_constants<T: Sized + Copy>(
        &mut self,
        pipeline: &ComputePipeline,
        offset: u32,
        constants: &T,
    ) {
        let array = [*constants];
        unsafe {
            self.device.handle().cmd_push_constants(
                self.handle(),
                *pipeline.layout(),
                ShaderStageFlags::COMPUTE,
                offset,
                std::slice::from_raw_parts(array.as_ptr() as *const u8, std::mem::size_of::<T>()),
            )
        }
    }
}

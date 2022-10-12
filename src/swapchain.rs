use crate::device_context::DeviceContext;
use crate::queue::CommandQueue;
use crate::swapchain_image::SwapchainImage;
use crate::swapchain_util::create_swapchain;
use ash::vk::{SurfaceKHR, SwapchainKHR};
use std::rc::Rc;

pub struct Swapchain {
    device: Rc<DeviceContext>,
    queue: Rc<CommandQueue>,
    swapchain_loader: ash::extensions::khr::Swapchain,
    surface: ash::vk::SurfaceKHR,
    handle: SwapchainKHR,
    images: Vec<SwapchainImage>,
    _image_views: Vec<ash::vk::ImageView>,
    present_semaphores: Vec<ash::vk::Semaphore>,
    renderpass: ash::vk::RenderPass,
    framebuffers: Vec<ash::vk::Framebuffer>,
    current_index: u32,
    format: ash::vk::Format,

    logical_width: u32,
    logical_height: u32,

    physical_width: u32,
    physical_height: u32,
}

impl Swapchain {
    pub fn new(
        device: Rc<DeviceContext>,
        surface: ash::vk::SurfaceKHR,
        old_swapchain: Option<&Swapchain>,
        queue: Rc<CommandQueue>,
        width: u32,
        height: u32,
    ) -> Self {
        let vulkan = device.gpu().vulkan();
        let surface_loader =
            ash::extensions::khr::Surface::new(vulkan.library(), vulkan.vk_instance());
        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(vulkan.vk_instance(), device.handle());
        let old_swapchain_handle = if let Some(old_sc) = old_swapchain {
            old_sc.handle()
        } else {
            SwapchainKHR::null()
        };
        let (swapchain, images, image_views, format, physical_width, physical_height) =
            create_swapchain(
                vulkan.vk_instance(),
                device.gpu().vk_physical_device(),
                device.handle(),
                &surface_loader,
                surface,
                &swapchain_loader,
                old_swapchain_handle,
                queue.clone(),
                width,
                height,
            );

        let attachments = [ash::vk::AttachmentDescription {
            format: format.format,
            samples: ash::vk::SampleCountFlags::TYPE_1,
            load_op: ash::vk::AttachmentLoadOp::DONT_CARE,
            store_op: ash::vk::AttachmentStoreOp::STORE,
            final_layout: ash::vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        }];

        let attachment_refs = [ash::vk::AttachmentReference {
            attachment: 0,
            layout: ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let dependencies = [ash::vk::SubpassDependency {
            src_subpass: ash::vk::SUBPASS_EXTERNAL,
            src_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: ash::vk::AccessFlags::COLOR_ATTACHMENT_READ
                | ash::vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_stage_mask: ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ..Default::default()
        }];

        let subpasses = [ash::vk::SubpassDescription::builder()
            .color_attachments(&attachment_refs)
            .pipeline_bind_point(ash::vk::PipelineBindPoint::GRAPHICS)
            .build()];

        let renderpass_create_info = ash::vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        let renderpass = unsafe {
            device
                .handle()
                .create_render_pass(&renderpass_create_info, None)
                .expect("Renderpass creation failed for swapchain")
        };

        let framebuffers: Vec<ash::vk::Framebuffer> = image_views
            .iter()
            .map(|&image_view| {
                let attachments = [image_view];
                let create_info = ash::vk::FramebufferCreateInfo::builder()
                    .render_pass(renderpass)
                    .attachments(&attachments)
                    .width(physical_width)
                    .height(physical_height)
                    .layers(1);
                unsafe {
                    device
                        .handle()
                        .create_framebuffer(&create_info, None)
                        .expect("Framebuffer creation failed for swapchain images")
                }
            })
            .collect();

        let semaphore_create_info = ash::vk::SemaphoreCreateInfo::default();

        let mut present_semaphores = Vec::new();
        for _ in 0..images.len() {
            present_semaphores.push(unsafe {
                device
                    .handle()
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap()
            });
        }

        let swapchain_images = images
            .iter()
            .map(|image| {
                SwapchainImage::new(
                    *image,
                    ash::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    format.format,
                    width,
                    height,
                )
            })
            .collect();

        Self {
            device: device.clone(),
            queue,
            surface,
            handle: swapchain,
            swapchain_loader,
            images: swapchain_images,
            _image_views: image_views,
            present_semaphores,
            renderpass,
            framebuffers,
            current_index: 0,
            format: format.format,
            logical_width: width,
            logical_height: height,
            physical_width,
            physical_height,
        }
    }

    pub fn surface(&self) -> &SurfaceKHR {
        &self.surface
    }

    pub fn handle(&self) -> SwapchainKHR {
        self.handle
    }

    pub fn next_frame_buffer(
        &mut self,
    ) -> Result<(bool, u32, ash::vk::Framebuffer, ash::vk::Semaphore), ash::vk::Result> {
        unsafe {
            let result = self.swapchain_loader.acquire_next_image(
                self.handle,
                std::u64::MAX,
                self.present_semaphores[self.current_index as usize],
                ash::vk::Fence::null(),
            );

            match result {
                Ok((index, sub_optimal)) => {
                    let result = (
                        sub_optimal,
                        index,
                        self.framebuffers[index as usize],
                        self.present_semaphores[index as usize],
                    );
                    self.current_index += 1;
                    self.current_index %= self.image_count() as u32;
                    Ok(result)
                }

                Err(code) => Err(code),
            }
        }
    }

    pub fn logical_width(&self) -> u32 {
        self.logical_width
    }

    pub fn logical_height(&self) -> u32 {
        self.logical_height
    }

    pub fn physical_width(&self) -> u32 {
        self.physical_width
    }

    pub fn physical_height(&self) -> u32 {
        self.physical_height
    }

    pub fn render_pass(&self) -> &ash::vk::RenderPass {
        &self.renderpass
    }

    pub fn semaphore(&self, index: usize) -> &ash::vk::Semaphore {
        &self.present_semaphores[index]
    }

    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    pub fn images(&self) -> &Vec<SwapchainImage> {
        &self.images
    }

    pub fn image(&self, index: u32) -> &SwapchainImage {
        &self.images[index as usize]
    }

    pub fn image_mut(&mut self, index: u32) -> &mut SwapchainImage {
        &mut self.images[index as usize]
    }

    pub fn format(&self) -> &ash::vk::Format {
        &self.format
    }

    pub fn swap(&self, semaphore: &ash::vk::Semaphore, index: u32) -> bool {
        let s = &[*semaphore];
        let sc = &[self.handle];
        let i = &[index];
        let present_info = ash::vk::PresentInfoKHR::builder()
            .wait_semaphores(s)
            .swapchains(sc)
            .image_indices(i);

        unsafe {
            let r = self
                .swapchain_loader
                .queue_present(self.queue.handle(), &present_info);

            r.is_err()
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for view in &self._image_views {
                self.device.handle().destroy_image_view(*view, None);
            }

            for semaphore in &self.present_semaphores {
                self.device.handle().destroy_semaphore(*semaphore, None);
            }

            for framebuffer in &self.framebuffers {
                self.device.handle().destroy_framebuffer(*framebuffer, None);
            }

            self.device
                .handle()
                .destroy_render_pass(self.renderpass, None);
        }
    }
}

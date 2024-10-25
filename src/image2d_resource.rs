use std::rc::Rc;

use crate::device_context::DeviceContext;
use crate::image_resource::ImageResource;
use crate::memory::memory_type_index;

use ash::vk::{
    DeviceMemory, Extent3D, Format, Image, ImageAspectFlags, ImageCreateInfo, ImageLayout,
    ImageSubresourceRange, ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo,
    ImageViewType, MemoryAllocateInfo, MemoryPropertyFlags, PhysicalDeviceMemoryProperties2,
    SampleCountFlags, SharingMode,
};

use ash::Device;

pub struct Image2DResource {
    device: Device,
    image: Image,
    memory: DeviceMemory,
    pub layout: ImageLayout,
    view: ImageView,
    width: u32,
    height: u32,
    format: Format,
}

impl Image2DResource {
    pub fn new(
        context: Rc<DeviceContext>,
        width: u32,
        height: u32,
        format: Format,
        usage: ImageUsageFlags,
        property_flags: MemoryPropertyFlags,
    ) -> Self {
        unsafe {
            let image_info = ImageCreateInfo::default()
                .image_type(ImageType::TYPE_2D)
                .samples(SampleCountFlags::TYPE_1)
                .sharing_mode(SharingMode::EXCLUSIVE)
                .format(format)
                .extent(Extent3D::default().width(width).height(height).depth(1))
                .array_layers(1)
                .mip_levels(1)
                .usage(usage);

            let device = context.handle();

            let image = device
                .create_image(&image_info, None)
                .expect("Image creation failed");
            let memory_requirements = device.get_image_memory_requirements(image);
            let mut properties = PhysicalDeviceMemoryProperties2::default();
            context.gpu().memory_properties(&mut properties);
            let type_index = memory_type_index(
                memory_requirements.memory_type_bits,
                &properties.memory_properties,
                property_flags,
            );
            if let Some(type_index) = type_index {
                let allocation_info = MemoryAllocateInfo::default()
                    .memory_type_index(type_index)
                    .allocation_size(memory_requirements.size);
                let memory = device
                    .allocate_memory(&allocation_info, None)
                    .expect("Memory allocation failed");

                device
                    .bind_image_memory(image, memory, 0)
                    .expect("Image memory bind failed");

                let subresource_range = ImageSubresourceRange::default()
                    .base_array_layer(0)
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1);
                let view_info = ImageViewCreateInfo::default()
                    .format(format)
                    .image(image)
                    .view_type(ImageViewType::TYPE_2D)
                    .subresource_range(subresource_range);
                let view = context
                    .handle()
                    .create_image_view(&view_info, None)
                    .expect("Image view creation failed");

                Self {
                    device: device.clone(),
                    image,
                    memory,
                    layout: ImageLayout::UNDEFINED,
                    width,
                    height,
                    format,
                    view,
                }
            } else {
                panic!()
            }
        }
    }

    pub fn new_device_local_storage_image(
        context: Rc<DeviceContext>,
        width: u32,
        height: u32,
        format: Format,
    ) -> Self {
        Self::new(
            context,
            width,
            height,
            format,
            ImageUsageFlags::STORAGE | ImageUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::DEVICE_LOCAL,
        )
    }
}

impl ImageResource for Image2DResource {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
    fn depth(&self) -> u32 {
        1
    }
    fn handle(&self) -> Image {
        self.image
    }
    fn format(&self) -> Format {
        self.format
    }
    fn layout(&self) -> ImageLayout {
        self.layout
    }

    fn set_layout(&mut self, layout: ImageLayout) {
        self.layout = layout
    }

    fn view(&self) -> ImageView {
        self.view
    }
}

impl Drop for Image2DResource {
    fn drop(&mut self) {
        unsafe { self.device.free_memory(self.memory, None) }
        unsafe { self.device.destroy_image(self.image, None) }
    }
}

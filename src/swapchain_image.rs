use ash::vk::{Format, Image, ImageLayout, ImageView};

use crate::image_resource::ImageResource;

pub struct SwapchainImage {
    handle: Image,
    layout: ImageLayout,
    format: Format,
    width: u32,
    height: u32,
    view: ImageView,
}

impl SwapchainImage {
    pub(crate) fn new(
        handle: Image,
        view: ImageView,
        layout: ImageLayout,
        format: Format,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            handle,
            layout,
            format,
            width,
            height,
            view,
        }
    }
}

impl ImageResource for SwapchainImage {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn depth(&self) -> u32 {
        1
    }

    fn format(&self) -> Format {
        self.format
    }

    fn set_layout(&mut self, layout: ImageLayout) {
        self.layout = layout
    }

    fn layout(&self) -> ImageLayout {
        self.layout
    }

    fn handle(&self) -> Image {
        self.handle
    }

    fn view(&self) -> ImageView {
        self.view
    }
}

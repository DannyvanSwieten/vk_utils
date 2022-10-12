use ash::vk::{Format, Image, ImageLayout};

pub trait ImageResource {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn depth(&self) -> u32;
    fn format(&self) -> Format;
    fn set_layout(&mut self, layout: ImageLayout);
    fn layout(&self) -> ImageLayout;
    fn handle(&self) -> Image;
}

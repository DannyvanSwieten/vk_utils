use ash::vk;
use vk_utils::vulkan::{self, Vulkan};

pub fn main() {
    let vulkan = Vulkan::new("My Application", &["VK_LAYER_KHRONOS_validation"], &[]);
}

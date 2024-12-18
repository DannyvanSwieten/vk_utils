use ash::ext::debug_utils;
use vk_utils::vulkan::Vulkan;

pub fn main() {
    let vulkan = Vulkan::new(
        "My Application",
        &["VK_LAYER_KHRONOS_validation"],
        &[debug_utils::NAME.to_str().unwrap()],
    );

    let physical_devices = vulkan.physical_devices();
    let graphics_device_index = physical_devices
        .iter()
        .position(|device| device.supports_graphics());

    let _logical_device = if let Some(index) = graphics_device_index {
        physical_devices[index].device_context(&[])
    } else {
        panic!()
    };
}

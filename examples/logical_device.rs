use vk_utils::vulkan::Vulkan;

pub fn main() {
    let vulkan = Vulkan::new(
        "My Application",
        &["VK_LAYER_KHRONOS_validation"],
        &["VK_EXT_debug_utils"],
    );

    let physical_devices = vulkan.physical_devices();
    let graphics_device_index = physical_devices
        .iter()
        .position(|device| device.supports_graphics());

    let logical_device = if let Some(index) = graphics_device_index {
        physical_devices[index].device_context(&[])
    } else {
        panic!()
    };
}

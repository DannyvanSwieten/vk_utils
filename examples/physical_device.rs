use vk_utils::{vulkan::Vulkan, DebugUtils};

pub fn main() {
    let vulkan = Vulkan::new(
        "My Application",
        &["VK_LAYER_KHRONOS_validation"],
        &[DebugUtils::name().to_str().unwrap()],
    );

    for device in vulkan.physical_devices() {
        println!("{}", device.name())
    }
}

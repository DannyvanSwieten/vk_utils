use std::panic;

use vk_utils::{vulkan::Vulkan, DebugUtils};

pub fn main() {
    let vulkan = Vulkan::new(
        "My Application",
        &["VK_LAYER_KHRONOS_validation"],
        &[DebugUtils::name().to_str().unwrap()],
    );

    let devices = vulkan.physical_devices();

    if devices.is_empty() {
        panic!("No physical devices found");
    } else {
        for device in devices {
            println!("{}", device.name())
        }
    }
}

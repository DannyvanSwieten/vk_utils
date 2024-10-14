use std::panic;

use ash::ext::debug_utils;
use vk_utils::vulkan::Vulkan;

pub fn main() {
    let vulkan = Vulkan::new(
        "My Application",
        &["VK_LAYER_KHRONOS_validation"],
        &[debug_utils::NAME.to_str().unwrap()],
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

use vk_utils::{vulkan::Vulkan, DebugUtils};

pub fn main() {
    let _vulkan = Vulkan::new(
        "My Application",
        &["VK_LAYER_KHRONOS_validation"],
        &[DebugUtils::name().to_str().unwrap()],
    );
}

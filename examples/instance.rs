use vk_utils::{vulkan::Vulkan, DebugUtils};

pub fn main() {
    let _vulkan = Vulkan::new(
        "My Application",
        &[],
        &[DebugUtils::name().to_str().unwrap()],
    );
}

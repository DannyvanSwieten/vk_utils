use vk_utils::vulkan::Vulkan;

pub fn main() {
    let _vulkan = Vulkan::new(
        "My Application",
        &["VK_LAYER_KHRONOS_validation"],
        &["VK_EXT_debug_utils"],
    );
}

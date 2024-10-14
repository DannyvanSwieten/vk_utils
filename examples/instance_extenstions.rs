use vk_utils::vulkan::Vulkan;

pub fn main() {
    let instance_extensions = Vulkan::available_instance_extensions();
    for extension in instance_extensions {
        println!("{}", extension);
    }
}

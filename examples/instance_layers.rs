use vk_utils::vulkan::Vulkan;

pub fn main() {
    let instance_layers = Vulkan::available_instance_layers();
    for layer in instance_layers {
        println!("{}", layer);
    }
}

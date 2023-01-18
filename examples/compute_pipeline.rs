use ash::vk::{BufferUsageFlags, MemoryPropertyFlags, QueueFlags};
use std::rc::Rc;
use vk_utils::buffer_resource::BufferResource;
use vk_utils::command_buffer::CommandBuffer;
use vk_utils::pipeline_descriptor::ComputePipeline;
use vk_utils::queue::CommandQueue;
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
        .position(|device| device.supports_compute());

    let logical_device = if let Some(index) = graphics_device_index {
        physical_devices[index].device_context(&[])
    } else {
        panic!()
    };

    let src = r"
    #version 450
    layout(set = 0, binding = 0) buffer Data{
        int x[];
    } data;
    void main(){
        uint i = gl_GlobalInvocationID.x;
        data.x[i] = data.x[i] * data.x[i];
    }
    ";

    let logical_device = Rc::new(logical_device);
    let pipeline = ComputePipeline::new_from_source_string(logical_device.clone(), 1, src, "main");
    let result = if let Some(mut pipeline) = pipeline {
        let data: Vec<i32> = (0..10).collect();
        let buffer_size = std::mem::size_of::<i32>() * data.len();
        let mut buffer = BufferResource::new(
            logical_device.clone(),
            buffer_size as _,
            MemoryPropertyFlags::HOST_VISIBLE,
            BufferUsageFlags::STORAGE_BUFFER,
        );
        buffer.upload(&data);
        pipeline.set_storage_buffer(0, 0, &buffer);
        let queue = Rc::new(CommandQueue::new(logical_device, QueueFlags::COMPUTE));
        let mut command_buffer = CommandBuffer::new(queue);
        command_buffer.begin();
        command_buffer.bind_compute_pipeline(&pipeline);
        command_buffer.dispatch_compute(data.len() as _, 1, 1);
        command_buffer.submit().wait();
        Some(buffer.copy_data::<i32>())
    } else {
        None
    };

    if let Some(result) = result {
        for i in result {
            println!("{}", i)
        }
    }
}

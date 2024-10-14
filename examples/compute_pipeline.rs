use ash::ext::debug_utils;
use ash::vk::QueueFlags;
use std::rc::Rc;
use vk_utils::buffer_resource::BufferResource;
use vk_utils::command_buffer::CommandBuffer;
use vk_utils::pipeline_descriptor::ComputePipeline;
use vk_utils::queue::CommandQueue;
use vk_utils::vulkan::Vulkan;

pub fn main() {
    let vulkan = Vulkan::new(
        "My Application",
        &[],
        &[debug_utils::NAME.to_str().unwrap()],
    );

    let logical_device =
        vulkan.devices_with_queue_support(QueueFlags::COMPUTE)[0].device_context(&[]);

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
    let queue = Rc::new(CommandQueue::new(
        logical_device.clone(),
        QueueFlags::COMPUTE,
    ));

    let pipeline =
        ComputePipeline::new_from_source_string(logical_device.clone(), 1, src, "main", None);
    let result = if let Some(mut pipeline) = pipeline {
        let data: Vec<i32> = (0..10).collect();
        let buffer = BufferResource::new_host_visible_with_data(logical_device.clone(), &data);
        pipeline.set_storage_buffer(0, 0, &buffer);

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

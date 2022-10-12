use std::rc::Rc;

use crate::device_context::DeviceContext;
use ash::vk::{CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, Queue, QueueFlags};

#[derive(Clone)]
pub struct CommandQueue {
    device: Rc<DeviceContext>,
    handle: Queue,
    queue_family_index: u32,
    command_pool: CommandPool,
}

impl CommandQueue {
    pub fn new(device: Rc<DeviceContext>, flags: QueueFlags) -> Self {
        let queue_family_index = device.queue_family_index(flags).unwrap();

        let pool_info = CommandPoolCreateInfo::builder()
            .flags(CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue_family_index)
            .build();
        let command_pool = unsafe {
            device
                .handle()
                .create_command_pool(&pool_info, None)
                .expect("Command Pool Creation failed")
        };
        Self {
            device: device.clone(),
            handle: device.queue(queue_family_index),
            queue_family_index,
            command_pool,
        }
    }

    pub fn family_type_index(&self) -> u32 {
        self.queue_family_index
    }

    pub fn handle(&self) -> Queue {
        self.handle
    }

    pub(crate) fn pool(&self) -> CommandPool {
        self.command_pool
    }
}

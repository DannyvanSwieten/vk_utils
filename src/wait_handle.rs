use ash::vk::Fence;

use crate::command_buffer::CommandBuffer;

pub struct WaitHandle {
    command_buffer: CommandBuffer,
    fence: Fence,
}

impl WaitHandle {
    pub(crate) fn new(command_buffer: CommandBuffer, fence: Fence) -> Self {
        Self {
            command_buffer,
            fence,
        }
    }

    pub fn has_completed(&self) -> bool {
        unsafe {
            match self
                .command_buffer
                .device()
                .handle()
                .wait_for_fences(&[self.fence], true, 0)
            {
                Err(_) => false,
                Ok(()) => true,
            }
        }
    }

    pub fn wait(&self) {
        unsafe {
            self.command_buffer
                .device()
                .handle()
                .wait_for_fences(&[self.fence], true, std::u64::MAX)
                .expect("Wait failed");
        }
    }

    pub fn wait_for(&self, timeout: u64) -> bool {
        unsafe {
            match self.command_buffer.device().handle().wait_for_fences(
                &[self.fence],
                true,
                timeout,
            ) {
                Err(_) => false,
                Ok(()) => true,
            }
        }
    }
}

impl Drop for WaitHandle {
    fn drop(&mut self) {
        unsafe {
            self.wait();
            self.command_buffer.device().handle().free_command_buffers(
                self.command_buffer.queue().pool(),
                &[self.command_buffer.handle()],
            )
        }
    }
}

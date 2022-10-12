use std::rc::Rc;

use crate::device_context::DeviceContext;
use crate::memory::memory_type_index;

use ash::vk::{
    Buffer, BufferCreateInfo, BufferDeviceAddressInfo, BufferUsageFlags, DeviceAddress,
    DeviceMemory, MappedMemoryRange, MemoryAllocateFlags, MemoryAllocateFlagsInfo,
    MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, SharingMode,
};
pub struct BufferResource {
    device: Rc<DeviceContext>,
    pub buffer: Buffer,
    memory: DeviceMemory,
    size: u64,
    content_size: u64,
}

impl BufferResource {
    pub fn upload<T>(&mut self, data: &[T]) {
        unsafe {
            let ptr = self
                .device
                .handle()
                .map_memory(self.memory, 0, self.size, MemoryMapFlags::default())
                .expect("Memory map failed on buffer");

            let size = self.content_size as usize / std::mem::size_of::<T>();

            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as _, size);

            let ranges = [*MappedMemoryRange::builder()
                .memory(self.memory)
                .size(self.size)];

            self.device
                .handle()
                .flush_mapped_memory_ranges(&ranges)
                .expect("Memory flush failed");
            self.device.handle().unmap_memory(self.memory);
        }
    }

    pub fn copy_aligned_to<T>(&mut self, data: &[T], element_size: Option<usize>, stride: usize) {
        unsafe {
            let element_size = if let Some(element_size) = element_size {
                element_size
            } else {
                std::mem::size_of::<T>()
            };

            let mut data_index = 0;
            for i in (0..self.content_size).step_by(stride) {
                let ptr = self
                    .device
                    .handle()
                    .map_memory(self.memory, i, stride as u64, MemoryMapFlags::default())
                    .expect("Memory map failed on buffer");

                std::ptr::copy_nonoverlapping(
                    data[data_index..data_index + element_size].as_ptr(),
                    ptr as *mut T,
                    element_size,
                );

                data_index += element_size;
                let ranges = [*MappedMemoryRange::builder()
                    .memory(self.memory)
                    .offset(i)
                    .size(ash::vk::WHOLE_SIZE)];

                self.device
                    .handle()
                    .flush_mapped_memory_ranges(&ranges)
                    .expect("Memory flush failed");
                self.device.handle().unmap_memory(self.memory);
            }
        }
    }

    pub fn copy_data<T: Copy>(&self) -> Vec<T> {
        unsafe {
            let ptr = self
                .device
                .handle()
                .map_memory(self.memory, 0, self.size, MemoryMapFlags::default())
                .expect("Memory map failed on buffer") as *mut T;

            let mut output = Vec::new();
            let count = (self.size as usize / std::mem::size_of::<T>()) as isize;
            for i in 0..count {
                output.push(*ptr.offset(i) as T);
            }

            output
        }
    }
}

impl BufferResource {
    pub fn new(
        device_context: Rc<DeviceContext>,
        size: u64,
        property_flags: MemoryPropertyFlags,
        usage: BufferUsageFlags,
    ) -> Self {
        unsafe {
            let device = device_context.handle();
            let buffer_info = BufferCreateInfo::builder()
                .size(size)
                .sharing_mode(SharingMode::EXCLUSIVE)
                .usage(usage);

            let buffer = device
                .create_buffer(&buffer_info, None)
                .expect("Buffer creation failed");
            let memory_requirements = device.get_buffer_memory_requirements(buffer);
            let type_index = memory_type_index(
                memory_requirements.memory_type_bits,
                &device_context.gpu().memory_properties().memory_properties,
                property_flags,
            );

            let mut flags = MemoryAllocateFlagsInfo::builder()
                .flags(MemoryAllocateFlags::DEVICE_ADDRESS_KHR)
                .build();
            if let Some(type_index) = type_index {
                let allocation_info = MemoryAllocateInfo::builder()
                    .push_next(&mut flags)
                    .memory_type_index(type_index)
                    .allocation_size(memory_requirements.size);
                let memory = device
                    .allocate_memory(&allocation_info, None)
                    .expect("Memory allocation failed");

                device
                    .bind_buffer_memory(buffer, memory, 0)
                    .expect("Buffer memory bind failed");

                Self {
                    device: device_context.clone(),
                    buffer,
                    memory,
                    size: memory_requirements.size,
                    content_size: size,
                }
            } else {
                panic!()
            }
        }
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn content_size(&self) -> u64 {
        self.content_size
    }

    pub fn device_address(&self) -> DeviceAddress {
        let v_address_info = BufferDeviceAddressInfo::builder().buffer(self.buffer);
        unsafe {
            self.device
                .handle()
                .get_buffer_device_address(&v_address_info)
        }
    }
}

impl Drop for BufferResource {
    fn drop(&mut self) {
        unsafe { self.device.handle().free_memory(self.memory, None) }
        unsafe { self.device.handle().destroy_buffer(self.buffer, None) }
    }
}

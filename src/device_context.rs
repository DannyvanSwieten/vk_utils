use crate::gpu::Gpu;
use ash::vk::{DeviceCreateInfo, DeviceQueueCreateInfo, QueueFlags};
use ash::Device;

pub struct DeviceContext {
    gpu: Gpu,
    handle: Device,
}

unsafe impl Send for DeviceContext {}

impl DeviceContext {
    pub(crate) fn new(gpu: &Gpu, extensions: &[&str], builder: DeviceCreateInfo) -> Self {
        let priorities: [f32; 1] = [1.];
        if let Some(index) = gpu.family_type_index(QueueFlags::GRAPHICS) {
            let queue_info = [DeviceQueueCreateInfo::default()
                .queue_priorities(&priorities)
                .queue_family_index(index)];

            let mut extension_names_raw: Vec<*const i8> = extensions
                .iter()
                .map(|layer_name| layer_name.as_ptr() as _)
                .collect();

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            {
                extension_names_raw.push(ash::khr::portability_subset::NAME.as_ptr());
            }

            let builder = builder
                .enabled_extension_names(&extension_names_raw)
                .queue_create_infos(&queue_info);

            unsafe {
                let device_context: Device = gpu
                    .vulkan()
                    .vk_instance()
                    .create_device(*gpu.vk_physical_device(), &builder, None)
                    .unwrap();
                Self {
                    gpu: gpu.clone(),
                    handle: device_context,
                }
            }
        } else {
            panic!("No queue family found");
        }
    }

    pub fn queue_family_index(&self, flags: QueueFlags) -> Option<u32> {
        self.gpu.family_type_index(flags)
    }

    pub fn queue(&self, queue_family_index: u32) -> ash::vk::Queue {
        unsafe { self.handle.get_device_queue(queue_family_index, 0) }
    }

    pub fn wait(&self) {
        unsafe {
            self.handle.device_wait_idle().expect("Wait failed");
        }
    }

    pub fn handle(&self) -> &Device {
        &self.handle
    }

    pub fn gpu(&self) -> &Gpu {
        &self.gpu
    }
}

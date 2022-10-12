use ash::vk::{
    DeviceCreateInfo, DeviceCreateInfoBuilder, ExtensionProperties, PhysicalDevice,
    PhysicalDeviceFeatures, PhysicalDeviceLimits, PhysicalDeviceMemoryProperties2,
    PhysicalDeviceProperties, PhysicalDeviceProperties2, PhysicalDeviceProperties2Builder,
    PhysicalDeviceType, QueueFamilyProperties, QueueFlags,
};

use crate::device_context::DeviceContext;
use crate::vulkan::Vulkan;
use std::ffi::CStr;

#[derive(Clone)]
pub struct Gpu {
    vulkan: Vulkan,
    physical_device: PhysicalDevice,
    features: PhysicalDeviceFeatures,
    properties: PhysicalDeviceProperties,
    memory_properties: PhysicalDeviceMemoryProperties2,
    queue_family_properties: Vec<QueueFamilyProperties>,
}

impl Gpu {
    pub(crate) fn new(vulkan: &Vulkan, physical_device: &PhysicalDevice) -> Self {
        unsafe {
            let features = vulkan
                .vk_instance()
                .get_physical_device_features(*physical_device);

            let properties = vulkan
                .vk_instance()
                .get_physical_device_properties(*physical_device);

            let mut memory_properties = PhysicalDeviceMemoryProperties2::default();
            vulkan
                .vk_instance()
                .get_physical_device_memory_properties2(*physical_device, &mut memory_properties);
            Self {
                vulkan: vulkan.clone(),
                features,
                properties,
                physical_device: *physical_device,
                queue_family_properties: vulkan
                    .vk_instance()
                    .get_physical_device_queue_family_properties(*physical_device),
                memory_properties,
            }
        }
    }

    pub fn device_context<'a, F>(
        &'a self,
        extensions: &[&'static CStr],
        builder_function: F,
    ) -> DeviceContext
    where
        F: FnOnce(DeviceCreateInfoBuilder<'a>) -> DeviceCreateInfoBuilder<'a>,
    {
        DeviceContext::new(
            self,
            extensions,
            builder_function(DeviceCreateInfo::builder()),
        )
    }

    pub(crate) fn family_type_index(&self, flags: QueueFlags) -> Option<u32> {
        for (index, queue_info) in self.queue_family_properties.iter().enumerate() {
            if queue_info.queue_flags.contains(flags) {
                return Some(index as u32);
            }
        }

        None
    }

    pub fn vk_physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }

    pub fn name(&self) -> String {
        let c_str = unsafe { CStr::from_ptr(self.properties.device_name.as_ptr()) };
        String::from(c_str.to_str().expect("String conversion failed"))
    }

    pub fn vendor_id(&self) -> u32 {
        self.properties.vendor_id
    }
    pub fn device_id(&self) -> u32 {
        self.properties.device_id
    }

    pub fn driver_version(&self) -> u32 {
        self.properties.driver_version
    }

    pub fn is_discrete(&self) -> bool {
        self.properties.device_type == PhysicalDeviceType::DISCRETE_GPU
    }

    pub fn is_virtual(&self) -> bool {
        self.properties.device_type == PhysicalDeviceType::VIRTUAL_GPU
    }

    pub fn limits(&self) -> PhysicalDeviceLimits {
        self.properties.limits
    }

    pub fn queue_family_count(&self) -> u32 {
        self.queue_family_properties.len() as u32
    }

    pub fn queue_count(&self, queue_family_index: u32) -> u32 {
        self.queue_family_properties[queue_family_index as usize].queue_count
    }

    pub fn device_extensions(&self) -> Vec<ExtensionProperties> {
        unsafe {
            self.vulkan()
                .vk_instance()
                .enumerate_device_extension_properties(self.physical_device)
                .expect("Device Extension enumeration failed")
        }
    }

    pub fn extension_properties<'a, F>(&self, builder_function: F) -> PhysicalDeviceProperties2
    where
        F: FnOnce(PhysicalDeviceProperties2Builder<'a>) -> PhysicalDeviceProperties2Builder<'a>,
    {
        let mut properties = builder_function(PhysicalDeviceProperties2::builder()).build();
        unsafe {
            self.vulkan()
                .vk_instance()
                .get_physical_device_properties2(self.physical_device, &mut properties);
        }

        properties
    }
    pub fn supports_graphics(&self) -> bool {
        for queue_info in self.queue_family_properties.iter() {
            if queue_info.queue_flags.contains(QueueFlags::GRAPHICS) {
                return true;
            }
        }

        false
    }

    pub fn supports_compute(&self) -> bool {
        for queue_info in self.queue_family_properties.iter() {
            if queue_info.queue_flags.contains(QueueFlags::COMPUTE) {
                return true;
            }
        }

        false
    }

    pub fn supports_transfer(&self) -> bool {
        for queue_info in self.queue_family_properties.iter() {
            if queue_info.queue_flags.contains(QueueFlags::TRANSFER) {
                return true;
            }
        }

        false
    }

    pub fn vulkan(&self) -> &Vulkan {
        &self.vulkan
    }

    pub fn memory_properties(&self) -> &PhysicalDeviceMemoryProperties2 {
        &self.memory_properties
    }
}

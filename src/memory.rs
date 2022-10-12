use ash::vk::{MemoryPropertyFlags, PhysicalDeviceMemoryProperties};

pub fn memory_type_index(
    filter: u32,
    properties: &PhysicalDeviceMemoryProperties,
    property_flags: MemoryPropertyFlags,
) -> Option<u32> {
    for i in 0..properties.memory_type_count {
        if (filter & (1 << i) > 1)
            && (properties.memory_types[i as usize].property_flags & property_flags)
                == property_flags
        {
            return Some(i);
        }
    }
    None
}

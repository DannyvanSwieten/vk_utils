use ash::vk::{
    make_api_version, ApplicationInfo, Bool32, DebugUtilsMessageSeverityFlagsEXT,
    DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCallbackDataEXT,
    DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT, InstanceCreateFlags,
    InstanceCreateInfo, QueueFlags, FALSE,
};
pub use ash::{Entry, Instance};
use std::borrow::Cow;
use std::ffi::{CStr, CString};

use ash::ext::{debug_utils, metal_surface};
use ash::khr::{get_physical_device_properties2, portability_enumeration, win32_surface};

use crate::gpu::Gpu;

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_type: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    FALSE
}

pub fn surface_extension_name() -> &'static CStr {
    if cfg!(unix) {
        metal_surface::NAME
    } else {
        win32_surface::NAME
    }
}

#[derive(Clone)]
pub struct Vulkan {
    _debug_callback: Option<DebugUtilsMessengerEXT>,
    library: Entry,
    instance: Instance,
}

impl Vulkan {
    pub fn new(name: &str, layers: &[&str], extensions: &[&str]) -> Self {
        let layers_names: Vec<String> = layers.iter().map(|s| s.to_string() + "\0").collect();
        let layers_names_raw: Vec<*const i8> =
            layers_names.iter().map(|s| s.as_ptr() as _).collect();
        let c_name = CString::new(name).unwrap();
        let appinfo = ApplicationInfo::default()
            .application_name(&c_name)
            .application_version(0)
            .engine_name(&c_name)
            .engine_version(0)
            .api_version(make_api_version(0, 1, 3, 0));

        let extension_names = extensions
            .iter()
            .map(|s| s.to_string() + "\0")
            .collect::<Vec<_>>();

        let mut extension_names_raw: Vec<*const i8> =
            extension_names.iter().map(|e| e.as_ptr() as _).collect();

        let mut flags = InstanceCreateFlags::default();
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            flags |= InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
            extension_names_raw.push(portability_enumeration::NAME.as_ptr());
            // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
            extension_names_raw.push(get_physical_device_properties2::NAME.as_ptr());
        }

        #[cfg(debug_assertions)]
        {
            // print all instance extensions
            let available_extensions = Self::available_instance_extensions();
            println!("Available Instance extensions:");
            for ext in available_extensions {
                println!("{}", ext);
            }
        }

        let create_info = InstanceCreateInfo::default()
            .application_info(&appinfo)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names_raw)
            .flags(flags);

        unsafe {
            let library = Entry::load().unwrap();
            let instance: Instance = library
                .create_instance(&create_info, None)
                .expect("Instance creation error");

            let debug_info = DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(DebugUtilsMessageTypeFlagsEXT::VALIDATION)
                .pfn_user_callback(Some(vulkan_debug_callback));

            if layers.contains(&"VK_LAYER_KHRONOS_validation") {
                println!("Validation layer enabled");
            }

            let debug_callback = if extension_names_raw.contains(&debug_utils::NAME.as_ptr()) {
                println!("Debug utils enabled");
                let debug_utils_loader = debug_utils::Instance::new(&library, &instance);
                match debug_utils_loader.create_debug_utils_messenger(&debug_info, None) {
                    Ok(succes) => Some(succes),
                    Err(error) => {
                        println!("{}", error);
                        None
                    }
                }
            } else {
                None
            };

            Self {
                _debug_callback: debug_callback,
                library,
                instance,
            }
        }
    }

    pub fn library(&self) -> &Entry {
        &self.library
    }
    pub fn vk_instance(&self) -> &Instance {
        &self.instance
    }

    pub fn devices_with_queue_support(&self, flags: QueueFlags) -> Vec<Gpu> {
        unsafe {
            self.instance
                .enumerate_physical_devices()
                .expect("Physical device error")
                .iter()
                .filter_map(|pdevice| {
                    self.instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .filter_map(|info| {
                            if info.queue_flags.contains(flags) {
                                Some(Gpu::new(self, pdevice))
                            } else {
                                None
                            }
                        })
                        .next()
                })
                .collect::<Vec<Gpu>>()
        }
    }

    pub fn physical_devices(&self) -> Vec<Gpu> {
        unsafe {
            self.instance
                .enumerate_physical_devices()
                .expect("Physical device enumeration failed")
                .iter()
                .map(|device| Gpu::new(self, device))
                .collect()
        }
    }

    pub fn available_instance_layers() -> Vec<String> {
        let library = unsafe { Entry::load().unwrap() };
        unsafe {
            library
                .enumerate_instance_layer_properties()
                .unwrap()
                .iter()
                .map(|layer| {
                    let name = CStr::from_ptr(layer.layer_name.as_ptr());
                    name.to_str().unwrap().to_string()
                })
                .collect()
        }
    }

    pub fn available_instance_extensions() -> Vec<String> {
        let library = unsafe { Entry::load().unwrap() };
        unsafe {
            library
                .enumerate_instance_extension_properties(None)
                .unwrap()
                .iter()
                .map(|ext| {
                    let name = CStr::from_ptr(ext.extension_name.as_ptr());
                    name.to_str().unwrap().to_string()
                })
                .collect()
        }
    }
}

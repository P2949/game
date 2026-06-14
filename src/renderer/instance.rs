//! Vulkan entry/instance creation plus validation-layer and debug-messenger
//! setup.

use ash::{Entry, vk};
use std::ffi::{CStr, CString};

use crate::renderer::debug;

/// Owns the Vulkan loader, instance, and optional debug messenger.
///
/// Keeping this as one RAII owner is important: if anything after instance
/// creation fails while the renderer is still being constructed, `Drop` tears
/// down the instance/debug messenger without relying on a fully-built
/// `VulkanContext`.
pub struct VulkanInstance {
    entry: Entry,
    instance: ash::Instance,
    debug_utils: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl VulkanInstance {
    pub fn new(window: &sdl3::video::Window) -> anyhow::Result<Self> {
        let entry = unsafe { Entry::load()? };

        let app_name = CString::new("sdl3-ash-demo")?;
        let engine_name = CString::new("no-engine")?;

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_3);

        // SDL tells you which platform-specific instance extensions are needed
        // to create a surface for this window.
        let sdl_extensions = window
            .vulkan_instance_extensions()
            .map_err(anyhow::Error::msg)?;

        let mut extension_names: Vec<CString> = sdl_extensions
            .iter()
            .map(|name| CString::new(name.as_str()).expect("SDL extension name contains NUL"))
            .collect();

        if cfg!(debug_assertions) {
            extension_names.push(ash::ext::debug_utils::NAME.to_owned());
        }

        let extension_ptrs: Vec<*const i8> =
            extension_names.iter().map(|name| name.as_ptr()).collect();

        let layer_names: Vec<&CStr> = if cfg!(debug_assertions) {
            if !validation_layer_available(&entry)? {
                anyhow::bail!(
                    "debug build requested {}, but the Vulkan validation layer is not installed",
                    debug::VALIDATION_LAYER.to_string_lossy()
                );
            }
            vec![debug::VALIDATION_LAYER]
        } else {
            vec![]
        };
        let layer_ptrs: Vec<*const i8> = layer_names.iter().map(|layer| layer.as_ptr()).collect();

        let validation_enables = [vk::ValidationFeatureEnableEXT::SYNCHRONIZATION_VALIDATION];
        let mut validation_features =
            vk::ValidationFeaturesEXT::default().enabled_validation_features(&validation_enables);

        let mut debug_create_info = debug::debug_messenger_create_info();

        let mut instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_ptrs)
            .enabled_layer_names(&layer_ptrs);

        if cfg!(debug_assertions) {
            instance_info = instance_info
                .push_next(&mut validation_features)
                .push_next(&mut debug_create_info);
        }

        let instance = unsafe { entry.create_instance(&instance_info, None)? };
        let mut owner = Self {
            entry,
            instance,
            debug_utils: None,
            debug_messenger: None,
        };

        if cfg!(debug_assertions) {
            let debug_utils = ash::ext::debug_utils::Instance::new(&owner.entry, &owner.instance);
            let messenger = unsafe {
                debug_utils
                    .create_debug_utils_messenger(&debug::debug_messenger_create_info(), None)?
            };
            owner.debug_utils = Some(debug_utils);
            owner.debug_messenger = Some(messenger);
        }

        Ok(owner)
    }

    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    pub fn handle(&self) -> &ash::Instance {
        &self.instance
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            if let (Some(debug_utils), Some(messenger)) = (&self.debug_utils, self.debug_messenger)
            {
                debug_utils.destroy_debug_utils_messenger(messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}

fn validation_layer_available(entry: &Entry) -> anyhow::Result<bool> {
    let layers = unsafe { entry.enumerate_instance_layer_properties()? };
    Ok(layers.iter().any(|layer| {
        let name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };
        name == debug::VALIDATION_LAYER
    }))
}

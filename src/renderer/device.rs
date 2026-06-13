use ash::vk;
use std::collections::HashSet;
use std::ffi::CStr;

#[derive(Debug, Clone, Copy)]
pub struct QueueFamilies {
    pub graphics: u32,
    pub present: u32,
}

impl QueueFamilies {
    pub fn unique_indices(self) -> Vec<u32> {
        let mut set = HashSet::new();
        set.insert(self.graphics);
        set.insert(self.present);
        let mut indices: Vec<_> = set.into_iter().collect();
        indices.sort_unstable();
        indices
    }
}

pub const REQUIRED_DEVICE_EXTENSIONS: &[&CStr] = &[ash::khr::swapchain::NAME];

pub fn find_queue_families(
    instance: &ash::Instance,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> anyhow::Result<Option<QueueFamilies>> {
    let families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut graphics = None;
    let mut present = None;
    let mut shared_graphics_present = None;

    for (index, family) in families.iter().enumerate() {
        let index = index as u32;
        let supports_present = unsafe {
            surface_loader.get_physical_device_surface_support(physical_device, index, surface)?
        };
        let supports_graphics = family.queue_flags.contains(vk::QueueFlags::GRAPHICS);

        if supports_graphics && supports_present {
            shared_graphics_present = Some(index);
            break;
        }

        if supports_graphics {
            graphics.get_or_insert(index);
        }

        if supports_present {
            present = Some(index);
        }
    }

    if let Some(index) = shared_graphics_present {
        return Ok(Some(QueueFamilies {
            graphics: index,
            present: index,
        }));
    }

    Ok(match (graphics, present) {
        (Some(graphics), Some(present)) => Some(QueueFamilies { graphics, present }),
        _ => None,
    })
}
pub fn supports_required_extensions(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> anyhow::Result<bool> {
    let available = unsafe { instance.enumerate_device_extension_properties(physical_device)? };

    for required in REQUIRED_DEVICE_EXTENSIONS {
        let found = available.iter().any(|ext| {
            let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            name == *required
        });

        if !found {
            return Ok(false);
        }
    }

    Ok(true)
}
pub fn supports_vulkan13_features(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> bool {
    let mut features13 = vk::PhysicalDeviceVulkan13Features::default();
    let mut features2 = vk::PhysicalDeviceFeatures2::default().push_next(&mut features13);

    unsafe {
        instance.get_physical_device_features2(physical_device, &mut features2);
    }

    features13.dynamic_rendering == vk::TRUE && features13.synchronization2 == vk::TRUE
}

#[derive(Debug)]
pub struct PhysicalDeviceSelection {
    pub physical_device: vk::PhysicalDevice,
    pub queue_families: QueueFamilies,
}

pub fn select_physical_device(
    instance: &ash::Instance,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> anyhow::Result<PhysicalDeviceSelection> {
    let devices = unsafe { instance.enumerate_physical_devices()? };

    let mut candidates = Vec::new();

    for physical_device in devices {
        let props = unsafe { instance.get_physical_device_properties(physical_device) };
        let name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) }
            .to_string_lossy()
            .into_owned();

        let Some(queue_families) =
            find_queue_families(instance, surface_loader, surface, physical_device)?
        else {
            log::info!("Skipping {name}: missing graphics/present queue");
            continue;
        };

        if !supports_required_extensions(instance, physical_device)? {
            log::info!("Skipping {name}: missing required device extension");
            continue;
        }

        if !supports_vulkan13_features(instance, physical_device) {
            log::info!("Skipping {name}: missing Vulkan 1.3 dynamic rendering/sync2 features");
            continue;
        }

        let mut score = match props.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 500,
            _ => 100,
        };

        if queue_families.graphics == queue_families.present {
            score += 250;
        }

        log::info!(
            "Candidate GPU: {name}, type={:?}, queues={queue_families:?}, unique_queues={:?}, score={score}",
            props.device_type,
            queue_families.unique_indices()
        );
        candidates.push((score, physical_device, queue_families, name));
    }

    candidates.sort_by_key(|(score, _, _, _)| *score);

    let Some((_score, physical_device, queue_families, name)) = candidates.pop() else {
        anyhow::bail!("No suitable Vulkan physical device found");
    };

    log::info!("Selected GPU: {name}");

    Ok(PhysicalDeviceSelection {
        physical_device,
        queue_families,
    })
}

pub struct LogicalDevice {
    pub device: ash::Device,
    #[allow(dead_code)]
    pub graphics_queue: vk::Queue,
    #[allow(dead_code)]
    pub present_queue: vk::Queue,
    #[allow(dead_code)]
    pub queues: QueueFamilies,
}

impl LogicalDevice {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        queues: QueueFamilies,
    ) -> anyhow::Result<Self> {
        let queue_priority = [1.0_f32];

        let queue_infos: Vec<_> = queues
            .unique_indices()
            .into_iter()
            .map(|queue_family_index| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(queue_family_index)
                    .queue_priorities(&queue_priority)
            })
            .collect();

        let extension_ptrs: Vec<*const i8> = REQUIRED_DEVICE_EXTENSIONS
            .iter()
            .map(|name| name.as_ptr())
            .collect();

        let mut features13 = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);

        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&extension_ptrs)
            .push_next(&mut features13);

        let device = unsafe { instance.create_device(physical_device, &device_info, None)? };

        let graphics_queue = unsafe { device.get_device_queue(queues.graphics, 0) };
        let present_queue = unsafe { device.get_device_queue(queues.present, 0) };

        Ok(Self {
            device,
            graphics_queue,
            present_queue,
            queues,
        })
    }
}

impl Drop for LogicalDevice {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

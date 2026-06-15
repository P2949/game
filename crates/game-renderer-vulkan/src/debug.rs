use ash::vk;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

pub const VALIDATION_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";

/// Receives Vulkan validation messages from the debug utils messenger.
///
/// # Safety
///
/// Vulkan calls this with `callback_data` either null or pointing to a valid
/// `vk::DebugUtilsMessengerCallbackDataEXT` for the duration of the callback.
/// The function only reads the message pointer when both pointers are non-null.
pub unsafe extern "system" fn vulkan_debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    ty: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let message = unsafe {
        if callback_data.is_null() || (*callback_data).p_message.is_null() {
            c"<null validation message>"
        } else {
            CStr::from_ptr((*callback_data).p_message as *const c_char)
        }
    };

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        log::warn!(
            "Vulkan validation [{severity:?} {ty:?}]: {}",
            message.to_string_lossy()
        );
    } else {
        log::debug!(
            "Vulkan validation [{severity:?} {ty:?}]: {}",
            message.to_string_lossy()
        );
    }

    vk::FALSE
}

pub fn debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static> {
    vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback))
}

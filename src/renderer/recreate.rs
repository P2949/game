//! Swapchain recreation decision policy.
//!
//! Kept free of Vulkan/SDL state so the rules can be unit-tested directly. The
//! context translates window/surface state into the booleans below and acts on
//! the returned [`SwapchainRecreateAction`].

/// Why a swapchain recreation is pending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapchainRecreateReason {
    /// Soft request from a resize or `SUBOPTIMAL_KHR`. May be skipped if the
    /// current swapchain extent already matches the window, and is subject to
    /// the resize debounce / recreate rate-limit.
    ResizeOrSuboptimal,
    /// Hard request from `ERROR_OUT_OF_DATE_KHR`. The current swapchain can no
    /// longer be used, so this must recreate as soon as a nonzero drawable
    /// extent is available, regardless of extent match or rate-limit.
    SurfaceOutOfDate,
}

/// The action the top-of-render gate should take for a pending recreate request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapchainRecreateAction {
    /// Drop the request and render this frame normally (extent already matches).
    ClearRequest,
    /// Skip rendering this frame and keep the request pending.
    Wait,
    /// Recreate the swapchain now.
    Recreate,
}

/// Pure decision policy for a pending swapchain recreate request.
///
/// Hard `SurfaceOutOfDate` requests recreate as soon as the drawable extent is
/// nonzero — they ignore both the extent-match short-circuit and the soft
/// rate-limit, since continuing to use an out-of-date swapchain is never valid.
pub fn swapchain_recreate_action(
    reason: SwapchainRecreateReason,
    has_nonzero_extent: bool,
    extent_matches: bool,
    soft_recreate_ready: bool,
) -> SwapchainRecreateAction {
    if !has_nonzero_extent {
        return SwapchainRecreateAction::Wait;
    }

    match reason {
        SwapchainRecreateReason::SurfaceOutOfDate => SwapchainRecreateAction::Recreate,
        SwapchainRecreateReason::ResizeOrSuboptimal if extent_matches => {
            SwapchainRecreateAction::ClearRequest
        }
        SwapchainRecreateReason::ResizeOrSuboptimal if soft_recreate_ready => {
            SwapchainRecreateAction::Recreate
        }
        SwapchainRecreateReason::ResizeOrSuboptimal => SwapchainRecreateAction::Wait,
    }
}

/// Records a soft recreate request without downgrading an already-pending hard
/// (`SurfaceOutOfDate`) request. Free function so the priority rule is testable.
pub fn request_soft_recreate(request: &mut Option<SwapchainRecreateReason>) {
    if *request != Some(SwapchainRecreateReason::SurfaceOutOfDate) {
        *request = Some(SwapchainRecreateReason::ResizeOrSuboptimal);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SwapchainRecreateAction, SwapchainRecreateReason, request_soft_recreate,
        swapchain_recreate_action,
    };

    #[test]
    fn soft_recreate_clears_when_extent_matches() {
        assert_eq!(
            swapchain_recreate_action(
                SwapchainRecreateReason::ResizeOrSuboptimal,
                true,
                true,
                false,
            ),
            SwapchainRecreateAction::ClearRequest
        );
    }

    #[test]
    fn soft_recreate_waits_when_extent_differs_but_rate_limited() {
        assert_eq!(
            swapchain_recreate_action(
                SwapchainRecreateReason::ResizeOrSuboptimal,
                true,
                false,
                false,
            ),
            SwapchainRecreateAction::Wait
        );
    }

    #[test]
    fn soft_recreate_recreates_when_extent_differs_and_ready() {
        assert_eq!(
            swapchain_recreate_action(
                SwapchainRecreateReason::ResizeOrSuboptimal,
                true,
                false,
                true,
            ),
            SwapchainRecreateAction::Recreate
        );
    }

    #[test]
    fn soft_recreate_waits_for_zero_size() {
        assert_eq!(
            swapchain_recreate_action(
                SwapchainRecreateReason::ResizeOrSuboptimal,
                false,
                false,
                true,
            ),
            SwapchainRecreateAction::Wait
        );
    }

    #[test]
    fn out_of_date_recreates_even_when_extent_matches() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::SurfaceOutOfDate, true, true, false),
            SwapchainRecreateAction::Recreate
        );
    }

    #[test]
    fn out_of_date_ignores_soft_rate_limit() {
        // Hard requests must recreate regardless of the soft rate-limit.
        assert_eq!(
            swapchain_recreate_action(
                SwapchainRecreateReason::SurfaceOutOfDate,
                true,
                false,
                false,
            ),
            SwapchainRecreateAction::Recreate
        );
    }

    #[test]
    fn out_of_date_waits_only_for_zero_size() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::SurfaceOutOfDate, false, true, true),
            SwapchainRecreateAction::Wait
        );
    }

    #[test]
    fn soft_request_sets_resize_reason_when_idle() {
        let mut request = None;
        request_soft_recreate(&mut request);
        assert_eq!(request, Some(SwapchainRecreateReason::ResizeOrSuboptimal));
    }

    #[test]
    fn soft_request_does_not_downgrade_out_of_date_request() {
        // The key regression guard: a benign soft request (resize/suboptimal)
        // must never clear a pending mandatory out-of-date recreate.
        let mut request = Some(SwapchainRecreateReason::SurfaceOutOfDate);
        request_soft_recreate(&mut request);
        assert_eq!(request, Some(SwapchainRecreateReason::SurfaceOutOfDate));
    }
}

//! Swapchain recreation decision policy.
//!
//! Kept free of Vulkan/SDL state so the rules can be unit-tested directly. The
//! context translates window/surface state into the booleans below and acts on
//! the returned [`SwapchainRecreateAction`].

/// Why a swapchain recreation is pending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapchainRecreateReason {
    /// Soft request from a resize. May be skipped if the current swapchain
    /// extent already matches the window, and is subject to the resize debounce
    /// / recreate rate-limit.
    Resize,
    /// Soft-but-real request from `SUBOPTIMAL_KHR`. Unlike a plain resize
    /// request, this is never cleared just because the current extent happens to
    /// match the window.
    Suboptimal,
    /// Hard request from `ERROR_OUT_OF_DATE_KHR`. The current swapchain can no
    /// longer be used, so this must recreate as soon as a nonzero drawable
    /// extent is available, regardless of extent match or rate-limit.
    SurfaceOutOfDate,
}

/// The action the top-of-render gate should take for a pending recreate request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapchainRecreateAction {
    /// Drop the request and render normally.
    ClearRequest,

    /// Rendering is not valid this frame, usually because the drawable extent is zero
    /// or the current swapchain is mandatory-out-of-date and cannot be used.
    SkipFrame,

    /// Keep the request pending but render this frame using the current swapchain.
    DeferAndRender,

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
        return SwapchainRecreateAction::SkipFrame;
    }

    match reason {
        SwapchainRecreateReason::SurfaceOutOfDate => SwapchainRecreateAction::Recreate,
        SwapchainRecreateReason::Resize if extent_matches => SwapchainRecreateAction::ClearRequest,
        SwapchainRecreateReason::Resize if soft_recreate_ready => SwapchainRecreateAction::Recreate,
        SwapchainRecreateReason::Resize => SwapchainRecreateAction::DeferAndRender,
        SwapchainRecreateReason::Suboptimal if soft_recreate_ready => {
            SwapchainRecreateAction::Recreate
        }
        SwapchainRecreateReason::Suboptimal => SwapchainRecreateAction::DeferAndRender,
    }
}

/// Records a soft recreate request without downgrading an already-pending hard
/// (`SurfaceOutOfDate`) request. Free function so the priority rule is testable.
pub fn request_soft_recreate(request: &mut Option<SwapchainRecreateReason>) {
    if *request != Some(SwapchainRecreateReason::SurfaceOutOfDate) {
        *request = Some(SwapchainRecreateReason::Resize);
    }
}

/// Records a suboptimal-present request without downgrading an already-pending
/// hard (`SurfaceOutOfDate`) request.
pub fn request_suboptimal_recreate(request: &mut Option<SwapchainRecreateReason>) {
    if *request != Some(SwapchainRecreateReason::SurfaceOutOfDate) {
        *request = Some(SwapchainRecreateReason::Suboptimal);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SwapchainRecreateAction, SwapchainRecreateReason, request_soft_recreate,
        request_suboptimal_recreate, swapchain_recreate_action,
    };

    #[test]
    fn resize_extent_matches_clears_request() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::Resize, true, true, false,),
            SwapchainRecreateAction::ClearRequest
        );
    }

    #[test]
    fn resize_rate_limited_defers_and_renders() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::Resize, true, false, false,),
            SwapchainRecreateAction::DeferAndRender
        );
    }

    #[test]
    fn soft_resize_defers_without_skipping_render_when_rate_limited() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::Resize, true, false, false),
            SwapchainRecreateAction::DeferAndRender
        );
    }

    #[test]
    fn resize_ready_recreates() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::Resize, true, false, true,),
            SwapchainRecreateAction::Recreate
        );
    }

    #[test]
    fn resize_zero_extent_skips_frame() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::Resize, false, false, true,),
            SwapchainRecreateAction::SkipFrame
        );
    }

    #[test]
    fn suboptimal_zero_extent_skips_frame() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::Suboptimal, false, false, true),
            SwapchainRecreateAction::SkipFrame
        );
    }

    #[test]
    fn out_of_date_zero_extent_skips_frame() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::SurfaceOutOfDate, false, true, true),
            SwapchainRecreateAction::SkipFrame
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
    fn out_of_date_recreates_even_when_rate_limited() {
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
    fn suboptimal_rate_limited_defers_and_renders() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::Suboptimal, true, true, false),
            SwapchainRecreateAction::DeferAndRender
        );
    }

    #[test]
    fn suboptimal_ready_recreates_even_when_extent_matches() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::Suboptimal, true, true, true),
            SwapchainRecreateAction::Recreate
        );
    }

    #[test]
    fn soft_request_sets_resize_reason_when_idle() {
        let mut request = None;
        request_soft_recreate(&mut request);
        assert_eq!(request, Some(SwapchainRecreateReason::Resize));
    }

    #[test]
    fn soft_request_does_not_downgrade_out_of_date_request() {
        // The key regression guard: a benign soft request (resize/suboptimal)
        // must never clear a pending mandatory out-of-date recreate.
        let mut request = Some(SwapchainRecreateReason::SurfaceOutOfDate);
        request_soft_recreate(&mut request);
        assert_eq!(request, Some(SwapchainRecreateReason::SurfaceOutOfDate));
    }

    #[test]
    fn suboptimal_ready_recreates_when_extent_differs() {
        assert_eq!(
            swapchain_recreate_action(SwapchainRecreateReason::Suboptimal, true, false, true),
            SwapchainRecreateAction::Recreate
        );
    }

    #[test]
    fn suboptimal_request_sets_suboptimal_reason_when_idle() {
        let mut request = None;
        request_suboptimal_recreate(&mut request);
        assert_eq!(request, Some(SwapchainRecreateReason::Suboptimal));
    }
}

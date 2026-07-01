use super::GameApp;

pub struct DebugOverlayAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
}

impl<'a, 'app> DebugOverlayAuthor<'a, 'app> {
    pub(super) fn new(app: &'a mut GameApp<'app>) -> Self {
        Self { app }
    }
}

impl DebugOverlayAuthor<'_, '_> {
    pub fn show_colliders(self) -> Self {
        self.app
            .configure_debug_overlay(|overlay| overlay.show_colliders = true);
        self
    }

    pub fn show_nav(self) -> Self {
        self.app
            .configure_debug_overlay(|overlay| overlay.show_nav = true);
        self
    }

    pub fn show_names(self) -> Self {
        self.app
            .configure_debug_overlay(|overlay| overlay.show_names = true);
        self
    }

    pub fn show_fps(self) -> Self {
        self.app
            .configure_debug_overlay(|overlay| overlay.show_fps = true);
        self
    }
}

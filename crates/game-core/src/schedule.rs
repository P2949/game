use crate::app::{Ctx, StartCtx};

pub type StartupSystem = Box<dyn for<'a> FnMut(&mut StartCtx<'a>) -> anyhow::Result<()>>;
pub type System = Box<dyn for<'a> FnMut(&mut Ctx<'a>, f32)>;

#[derive(Default)]
pub struct Schedule {
    startup: Vec<StartupSystem>,
    fixed: Vec<System>,
    update: Vec<System>,
    render_extract: Vec<System>,
    ui: Vec<System>,
    fixed_pause_guarded: bool,
}

impl Schedule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_startup(
        &mut self,
        system: impl for<'a> FnMut(&mut StartCtx<'a>) -> anyhow::Result<()> + 'static,
    ) -> &mut Self {
        self.startup.push(Box::new(system));
        self
    }

    pub fn add_fixed(
        &mut self,
        system: impl for<'a> FnMut(&mut Ctx<'a>, f32) + 'static,
    ) -> &mut Self {
        self.fixed.push(Box::new(system));
        self
    }

    pub fn add_update(
        &mut self,
        system: impl for<'a> FnMut(&mut Ctx<'a>, f32) + 'static,
    ) -> &mut Self {
        self.update.push(Box::new(system));
        self
    }

    pub fn add_render_extract(
        &mut self,
        system: impl for<'a> FnMut(&mut Ctx<'a>, f32) + 'static,
    ) -> &mut Self {
        self.render_extract.push(Box::new(system));
        self
    }

    pub fn add_ui(&mut self, system: impl for<'a> FnMut(&mut Ctx<'a>, f32) + 'static) -> &mut Self {
        self.ui.push(Box::new(system));
        self
    }

    pub fn mark_fixed_pause_guarded(&mut self) -> &mut Self {
        self.fixed_pause_guarded = true;
        self
    }

    pub fn has_startup_systems(&self) -> bool {
        !self.startup.is_empty()
    }

    pub fn has_frame_systems(&self) -> bool {
        !(self.fixed.is_empty()
            && self.update.is_empty()
            && self.render_extract.is_empty()
            && self.ui.is_empty())
    }

    pub fn has_render_extract_systems(&self) -> bool {
        !self.render_extract.is_empty()
    }

    pub fn has_ui_systems(&self) -> bool {
        !self.ui.is_empty()
    }

    pub fn has_fixed_systems(&self) -> bool {
        !self.fixed.is_empty()
    }

    pub fn fixed_pause_guarded(&self) -> bool {
        self.fixed_pause_guarded
    }

    pub fn run_startup(&mut self, ctx: &mut StartCtx<'_>) -> anyhow::Result<()> {
        for system in &mut self.startup {
            system(ctx)?;
        }
        Ok(())
    }

    pub fn run_fixed(&mut self, ctx: &mut Ctx<'_>, dt: f32) {
        for system in &mut self.fixed {
            system(ctx, dt);
        }
    }

    pub fn run_update(&mut self, ctx: &mut Ctx<'_>, dt: f32) {
        for system in &mut self.update {
            system(ctx, dt);
        }
    }

    pub fn run_render_extract(&mut self, ctx: &mut Ctx<'_>, dt: f32) {
        for system in &mut self.render_extract {
            system(ctx, dt);
        }
    }

    pub fn run_ui(&mut self, ctx: &mut Ctx<'_>, dt: f32) {
        for system in &mut self.ui {
            system(ctx, dt);
        }
    }

    pub fn run_frame(&mut self, ctx: &mut Ctx<'_>, dt: f32) {
        self.run_fixed(ctx, dt);
        self.run_update(ctx, dt);
        self.run_render_extract(ctx, dt);
        self.run_ui(ctx, dt);
    }
}

pub struct ScheduleValidator<'a> {
    schedule: &'a Schedule,
    start_map_set: bool,
    builtin_render_extract: bool,
}

impl<'a> ScheduleValidator<'a> {
    pub fn new(schedule: &'a Schedule) -> Self {
        Self {
            schedule,
            start_map_set: false,
            builtin_render_extract: false,
        }
    }

    pub fn start_map_set(mut self, start_map_set: bool) -> Self {
        self.start_map_set = start_map_set;
        self
    }

    /// Declares that the host runtime always populates the render frame itself
    /// (e.g. by extracting tilemap/entity sprites every frame), so the schedule
    /// is not required to register its own `render_extract` system. Content that
    /// runs on a runtime without built-in extraction must still register one.
    pub fn builtin_render_extract(mut self) -> Self {
        self.builtin_render_extract = true;
        self
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.start_map_set {
            anyhow::bail!("schedule validation failed: start map is not set");
        }
        if !self.builtin_render_extract && !self.schedule.has_render_extract_systems() {
            anyhow::bail!(
                "schedule validation failed: at least one render extraction system is required"
            );
        }
        if self.schedule.has_fixed_systems() && !self.schedule.fixed_pause_guarded() {
            anyhow::bail!("schedule validation failed: fixed systems must be marked pause-guarded");
        }
        if !self.schedule.has_ui_systems() {
            anyhow::bail!("schedule validation failed: at least one UI system is required");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{Ctx, RenderFrame, StartCtx};
    use crate::audio::{Audio, AudioCommands};
    use crate::camera::Camera2D;
    use crate::gfx::Gfx;
    use crate::input::{FrameActions, Input};
    use crate::world::World;

    use super::Schedule;

    #[test]
    fn schedule_runs_startup_and_frame_stages_in_order() {
        let mut schedule = Schedule::new();
        schedule.add_startup(|ctx| {
            ctx.world.insert_resource(Vec::<&'static str>::new());
            Ok(())
        });
        schedule.add_fixed(|ctx, _| {
            ctx.world
                .get_resource_mut::<Vec<&'static str>>()
                .unwrap()
                .push("fixed");
        });
        schedule.add_update(|ctx, _| {
            ctx.world
                .get_resource_mut::<Vec<&'static str>>()
                .unwrap()
                .push("update");
        });
        schedule.add_render_extract(|ctx, _| {
            ctx.world
                .get_resource_mut::<Vec<&'static str>>()
                .unwrap()
                .push("render_extract");
        });
        schedule.add_ui(|ctx, _| {
            ctx.world
                .get_resource_mut::<Vec<&'static str>>()
                .unwrap()
                .push("ui");
        });
        schedule.mark_fixed_pause_guarded();

        let mut world = World::new();
        let mut map_slot = None;
        schedule
            .run_startup(&mut StartCtx::new(&mut world, &mut map_slot))
            .unwrap();

        let mut camera = Camera2D::new(glam::Vec2::ZERO, 1.0);
        let mut frame = RenderFrame::new(camera);
        let mut audio_commands = AudioCommands::default();
        let map = crate::tilemap::TileMap::from_rows(&["."], 10.0);
        let nav = crate::nav::NavGrid::from_tilemap(&map);
        let input = Input::new(glam::Vec2::ZERO, 0.0, FrameActions::default());
        let mut ctx = Ctx {
            world: &mut world,
            map: &map,
            nav: &nav,
            input: &input,
            camera: &mut camera,
            gfx: Gfx::new(&mut frame),
            audio: Audio::new(&mut audio_commands),
        };
        schedule.run_frame(&mut ctx, 1.0 / 120.0);

        assert_eq!(
            world.get_resource::<Vec<&'static str>>().unwrap(),
            &vec!["fixed", "update", "render_extract", "ui"]
        );
    }

    #[test]
    fn schedule_validator_requires_start_map_render_ui_and_pause_guard() {
        let mut schedule = Schedule::new();
        schedule.add_fixed(|_, _| {});
        schedule.add_render_extract(|_, _| {});
        schedule.add_ui(|_, _| {});
        schedule.mark_fixed_pause_guarded();

        super::ScheduleValidator::new(&schedule)
            .start_map_set(true)
            .validate()
            .unwrap();
    }

    #[test]
    fn schedule_validator_reports_each_missing_requirement() {
        // Missing start map.
        let mut schedule = Schedule::new();
        schedule.add_render_extract(|_, _| {});
        schedule.add_ui(|_, _| {});
        let err = super::ScheduleValidator::new(&schedule)
            .validate()
            .unwrap_err();
        assert!(err.to_string().contains("start map"));

        // Missing render extraction with no built-in extraction declared.
        let mut schedule = Schedule::new();
        schedule.add_ui(|_, _| {});
        let err = super::ScheduleValidator::new(&schedule)
            .start_map_set(true)
            .validate()
            .unwrap_err();
        assert!(err.to_string().contains("render extraction"));

        // Unguarded fixed systems.
        let mut schedule = Schedule::new();
        schedule.add_fixed(|_, _| {});
        schedule.add_ui(|_, _| {});
        let err = super::ScheduleValidator::new(&schedule)
            .start_map_set(true)
            .builtin_render_extract()
            .validate()
            .unwrap_err();
        assert!(err.to_string().contains("pause-guarded"));

        // Missing UI system.
        let mut schedule = Schedule::new();
        schedule.add_render_extract(|_, _| {});
        let err = super::ScheduleValidator::new(&schedule)
            .start_map_set(true)
            .validate()
            .unwrap_err();
        assert!(err.to_string().contains("UI system"));
    }

    #[test]
    fn schedule_validator_accepts_builtin_render_extract_without_extract_system() {
        // A runtime that extracts sprites itself satisfies the render requirement
        // even when the schedule registers no `render_extract` system.
        let mut schedule = Schedule::new();
        schedule.add_fixed(|_, _| {});
        schedule.add_ui(|_, _| {});
        schedule.mark_fixed_pause_guarded();

        super::ScheduleValidator::new(&schedule)
            .start_map_set(true)
            .builtin_render_extract()
            .validate()
            .unwrap();
    }
}

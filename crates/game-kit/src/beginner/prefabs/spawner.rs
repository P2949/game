use super::shared::*;

pub struct SpawnerPrefabAuthor<'a, 'app> {
    app: &'a mut GameApp<'app>,
    name: String,
    display_name: Option<String>,
    prefab: Option<String>,
    every_seconds: f32,
    max_alive: Option<usize>,
    placement: SpawnPlacement,
}

impl<'a, 'app> SpawnerPrefabAuthor<'a, 'app> {
    pub(crate) fn new(app: &'a mut GameApp<'app>, name: String) -> Self {
        Self {
            app,
            name,
            display_name: None,
            prefab: None,
            every_seconds: 1.0,
            max_alive: None,
            placement: SpawnPlacement::AtSpawner,
        }
    }

    pub fn named(mut self, display_name: impl IntoContentName) -> Self {
        self.display_name = Some(display_name.into_content_name());
        self
    }

    pub fn spawn(mut self, prefab: impl Into<String>) -> Self {
        self.prefab = Some(prefab.into());
        self
    }

    pub fn every_seconds(mut self, seconds: f32) -> Self {
        self.every_seconds = seconds.max(0.001);
        self
    }

    pub fn max_alive(mut self, max_alive: usize) -> Self {
        self.max_alive = Some(max_alive);
        self
    }

    pub fn at_spawner(mut self) -> Self {
        self.placement = SpawnPlacement::AtSpawner;
        self
    }

    pub fn near_player(mut self, radius: f32) -> Self {
        self.placement = SpawnPlacement::NearPlayer {
            radius: radius.max(0.0),
        };
        self
    }

    pub fn at_first_floor(mut self) -> Self {
        self.placement = SpawnPlacement::AtFirstFloor;
        self
    }

    pub fn build(self) -> Result<()> {
        let spawn_prefab = self.prefab.ok_or_else(|| {
            anyhow::anyhow!(
                "spawner prefab '{}' does not name a prefab to spawn.\n\nAdd:\n    .spawn(\"slime\")",
                self.name
            )
        })?;
        let prefab_name = self.name.clone();
        let display_name = self.display_name.unwrap_or_else(|| prefab_name.clone());
        let every_seconds = self.every_seconds;
        let max_alive = self.max_alive;
        let placement = self.placement;

        self.app.prefab(self.name, move |prefab| {
            prefab
                .spawn(move |at| {
                    (
                        Name::new(display_name.clone()),
                        Transform::at(at),
                        Spawner {
                            prefab: spawn_prefab.clone(),
                            every_seconds,
                            timer: 0.0,
                            max_alive,
                            placement: placement.clone(),
                        },
                    )
                })?
                .require::<Transform>()
                .require::<Spawner>();
            Ok(())
        })
    }
}

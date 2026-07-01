use super::*;

pub(super) fn build_audio(game: &mut GameApp<'_>, audio: AudioFile) {
    let initial_audio = audio;
    let mut state = RuntimeAudioState::default();
    game.update(move |game: &mut GameCtx<'_, '_>, _dt| {
        let audio = game
            .resource::<BeginnerRuntimeConfig>()
            .map(|config| config.audio().clone())
            .unwrap_or_else(|| initial_audio.clone());
        apply_runtime_audio(game, &audio, &mut state);
    });
}

#[derive(Clone, Debug, Default, PartialEq)]
struct RuntimeAudioState {
    volumes: Option<(Option<f32>, Option<f32>, Option<f32>)>,
    active_music: Option<RuntimeMusicPlayback>,
}

#[derive(Clone, Debug, PartialEq)]
struct RuntimeMusicPlayback {
    scene: String,
    playback: MusicPlaybackFile,
}

fn apply_runtime_audio(
    game: &mut GameCtx<'_, '_>,
    audio: &AudioFile,
    state: &mut RuntimeAudioState,
) {
    let volumes = (audio.master_volume, audio.music_volume, audio.sfx_volume);
    if state.volumes != Some(volumes) {
        if let Some(volume) = audio.master_volume {
            game.audio().set_master_volume(volume);
        }
        if let Some(volume) = audio.music_volume {
            game.audio().set_music_volume(volume);
        }
        if let Some(volume) = audio.sfx_volume {
            game.audio().set_sfx_volume(volume);
        }
        state.volumes = Some(volumes);
    }

    let Some(scene) = game.current_scene_name() else {
        return;
    };
    let Some(playback) = audio.music_on_scene.get(&scene).cloned() else {
        return;
    };
    let requested = RuntimeMusicPlayback { scene, playback };
    if state.active_music.as_ref() == Some(&requested) {
        return;
    }

    let music = game
        .audio()
        .play_music(&requested.playback.track)
        .volume(requested.playback.volume);
    if let Some(fade) = requested.playback.fade_in {
        music.fade_in(fade);
    }
    state.active_music = Some(requested);
}

pub(super) fn build_actions(
    game: &mut GameApp<'_>,
    actions: Vec<BeginnerActionFile>,
    controls: TopDownControls,
) {
    for (index, action) in actions.into_iter().enumerate() {
        match action {
            BeginnerActionFile::PlayerShoots(shoot) => {
                register_runtime_player_shoots_action(game, index, shoot, controls);
            }
        }
    }
}

fn register_runtime_player_shoots_action(
    game: &mut GameApp<'_>,
    index: usize,
    initial: PlayerShootsFile,
    controls: TopDownControls,
) {
    let action = initial.action.resolve(controls);
    let mut cooldown: f32 = 0.0;
    game.fixed(move |game: &mut GameCtx<'_, '_>, dt: f32| {
        cooldown = (cooldown - dt).max(0.0);
        let shoot = game
            .resource::<BeginnerRuntimeConfig>()
            .and_then(|config| config.player_shoots_action(index))
            .cloned()
            .unwrap_or_else(|| initial.clone());
        if cooldown == 0.0 && game.pressed(action) {
            cooldown = shoot.cooldown.max(0.0);
            fire_runtime_player_shot(game, &shoot);
        }
    });
}

fn fire_runtime_player_shot(game: &mut GameCtx<'_, '_>, shoot: &PlayerShootsFile) {
    let fired = match shoot.direction {
        ShotDirectionFile::TowardsMouse => {
            game.player().shoot(shoot.prefab.clone()).towards_mouse()
        }
        ShotDirectionFile::Right => game.player().shoot(shoot.prefab.clone()).right(),
        ShotDirectionFile::Left => game.player().shoot(shoot.prefab.clone()).left(),
        ShotDirectionFile::Up => game.player().shoot(shoot.prefab.clone()).up(),
        ShotDirectionFile::Down => game.player().shoot(shoot.prefab.clone()).down(),
    };
    if let Some(sound) = &shoot.sound {
        fired.play_sound_named(sound);
    }
}

pub(super) fn build_custom_rules(game: &mut GameApp<'_>, custom_rules: Vec<CustomRuleFile>) {
    for custom_rule in custom_rules {
        match custom_rule {
            CustomRuleFile::Countdown(rule) => {
                register_runtime_countdown_rule(game, rule.name);
            }
        }
    }
}

#[derive(Default)]
struct RuleConditionRuntime {
    elapsed: f32,
    fired: bool,
}

pub(super) fn build_script_rule(
    game: &mut GameApp<'_>,
    rule: BeginnerScriptRuleFile,
    controls: TopDownControls,
) {
    match rule {
        BeginnerScriptRuleFile::When { condition, effects } => {
            let mut runtime = RuleConditionRuntime::default();
            game.every_tick(move |game, dt| {
                let active = script_condition_active(game, &condition, controls, dt, &mut runtime);
                if active && !runtime.fired {
                    apply_game_rule_effects(game, &effects);
                    runtime.fired = true;
                } else if !active {
                    runtime.fired = false;
                }
            });
        }
        BeginnerScriptRuleFile::OnEnemyDeath { prefab, effects } => {
            game.on_enemy_death_event(move |event| {
                let matches = {
                    let enemy = event.enemy();
                    enemy.is_prefab(&prefab)
                };
                if matches {
                    apply_enemy_death_rule_effects(event, &effects);
                }
            });
        }
        BeginnerScriptRuleFile::EverySeconds { seconds, effects } => {
            game.every_seconds(seconds, move |game| {
                apply_game_rule_effects(game, &effects);
            });
        }
        BeginnerScriptRuleFile::OnScoreReaches { score, effects } => {
            game.on_score_reaches(score, move |game| {
                apply_game_rule_effects(game, &effects);
            });
        }
    }
}

fn script_condition_active(
    game: &mut BeginnerGame<'_, '_, '_>,
    condition: &RuleConditionFile,
    controls: TopDownControls,
    dt: f32,
    runtime: &mut RuleConditionRuntime,
) -> bool {
    match condition {
        RuleConditionFile::AllEnemiesDead => game.enemies().alive().count() == 0,
        RuleConditionFile::AllPickupsCollected => game.pickups().alive().count() == 0,
        RuleConditionFile::ScoreAtLeast(score) => game.score().value() >= *score,
        RuleConditionFile::PlayerHealthBelow(health) => game
            .player()
            .health()
            .is_some_and(|current| current < *health),
        RuleConditionFile::TimerReached { seconds, .. } => {
            runtime.elapsed += dt.max(0.0);
            runtime.elapsed >= seconds.max(0.0)
        }
        RuleConditionFile::MapIs(map) => game.current_map_name().as_deref() == Some(map.as_str()),
        RuleConditionFile::SceneIs(scene) => {
            game.current_scene_name().as_deref() == Some(scene.as_str())
        }
        RuleConditionFile::TagCountZero(tag) => game.actors_tagged(tag).alive().count() == 0,
        RuleConditionFile::ActionPressed(action) => game.pressed(action.resolve(controls)),
    }
}

fn apply_game_rule_effects(game: &mut BeginnerGame<'_, '_, '_>, effects: &[RuleEffectFile]) {
    for effect in effects {
        match effect {
            RuleEffectFile::AddScore(amount) => game.score().add(*amount),
            RuleEffectFile::SetScore(score) => game.score().set(*score),
            RuleEffectFile::DamageTagged { tag, amount, .. } => {
                game.actors_tagged(tag).damage(*amount);
            }
            RuleEffectFile::DamagePlayer { amount, .. } => {
                game.player().damage(*amount);
            }
            RuleEffectFile::DespawnSelf => {}
            RuleEffectFile::PlaySound(key) => game.play_sound_named(key),
            RuleEffectFile::PlayMusic(key) => game.play_music_named(key),
            RuleEffectFile::StopMusic => game.audio().stop_music(),
            RuleEffectFile::SpawnPrefab(prefab) => {
                game.spawn(prefab.clone()).at_first_floor();
            }
            RuleEffectFile::SpawnNearPlayer { prefab, radius } => {
                game.spawn(prefab.clone()).near_player(*radius);
            }
            RuleEffectFile::ChangeScene(scene) => game.change_scene_or_log(scene),
            RuleEffectFile::ChangeMap(map) => game.change_map_or_log(map),
            RuleEffectFile::RestartCurrentMap => game.restart_current_map_or_log(),
            RuleEffectFile::ShowUiText(text) => game.show_rule_text(text),
            RuleEffectFile::HealPlayer(amount) => {
                game.player().heal(*amount);
            }
            RuleEffectFile::SetData { tag, key, value } => {
                game.actors_tagged(tag).set_data(key, *value);
            }
            RuleEffectFile::DespawnTagged(tag) => {
                game.actors_tagged(tag).despawn();
            }
        }
    }
}

fn apply_enemy_death_rule_effects(
    event: &mut EnemyDeathEvent<'_, '_, '_>,
    effects: &[RuleEffectFile],
) {
    let position = event.enemy_position();
    for effect in effects {
        match effect {
            RuleEffectFile::AddScore(amount) => event.score().add(*amount),
            RuleEffectFile::SetScore(score) => event.score().set(*score),
            RuleEffectFile::DespawnSelf => {
                event.enemy().despawn();
            }
            RuleEffectFile::PlaySound(key) => event.play_sound(key),
            RuleEffectFile::SpawnPrefab(prefab) => {
                if let Some(position) = position {
                    event.spawn(prefab.clone()).at_world(position);
                }
            }
            RuleEffectFile::SpawnNearPlayer { prefab, radius } => {
                event.spawn(prefab.clone()).near_player(*radius);
            }
            RuleEffectFile::ChangeScene(scene) => event.change_scene(scene),
            RuleEffectFile::ChangeMap(map) => event.change_map(map),
            RuleEffectFile::DamageTagged { .. }
            | RuleEffectFile::DamagePlayer { .. }
            | RuleEffectFile::PlayMusic(_)
            | RuleEffectFile::StopMusic
            | RuleEffectFile::RestartCurrentMap
            | RuleEffectFile::ShowUiText(_)
            | RuleEffectFile::HealPlayer(_)
            | RuleEffectFile::SetData { .. }
            | RuleEffectFile::DespawnTagged(_) => {}
        }
    }
}

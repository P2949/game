pub use crate::backend::AudioCommand;
use crate::backend::SoundHandle;

#[derive(Default)]
pub struct AudioCommands {
    commands: Vec<AudioCommand>,
}

impl AudioCommands {
    pub fn push(&mut self, command: AudioCommand) {
        self.commands.push(command);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = AudioCommand> + '_ {
        self.commands.drain(..)
    }
}

pub struct Audio<'a> {
    commands: &'a mut AudioCommands,
}

impl<'a> Audio<'a> {
    pub fn new(commands: &'a mut AudioCommands) -> Self {
        Self { commands }
    }

    pub fn play(&mut self, sound: SoundHandle, volume: f32) {
        self.commands.push(AudioCommand::Play {
            sound,
            volume,
            looping: false,
            bus: None,
        });
    }

    pub fn play_on_bus(&mut self, sound: SoundHandle, volume: f32, bus: impl Into<String>) {
        self.commands.push(AudioCommand::Play {
            sound,
            volume,
            looping: false,
            bus: Some(bus.into()),
        });
    }

    pub fn play_music(&mut self, sound: SoundHandle, volume: f32) {
        self.commands.push(AudioCommand::PlayMusic {
            sound,
            volume,
            fade_in_seconds: None,
        });
    }

    pub fn play_music_fade_in(&mut self, sound: SoundHandle, volume: f32, duration_seconds: f32) {
        self.commands.push(AudioCommand::PlayMusic {
            sound,
            volume,
            fade_in_seconds: Some(duration_seconds),
        });
    }

    pub fn stop_music(&mut self) {
        self.commands.push(AudioCommand::StopMusic);
    }

    pub fn crossfade_music(&mut self, sound: SoundHandle, volume: f32, duration_seconds: f32) {
        self.commands.push(AudioCommand::CrossfadeMusic {
            sound,
            volume,
            duration_seconds,
        });
    }

    pub fn pause_music(&mut self) {
        self.commands.push(AudioCommand::PauseMusic);
    }

    pub fn resume_music(&mut self) {
        self.commands.push(AudioCommand::ResumeMusic);
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.commands.push(AudioCommand::SetMasterVolume { volume });
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.commands.push(AudioCommand::SetSfxVolume { volume });
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.commands.push(AudioCommand::SetMusicVolume { volume });
    }

    pub fn set_bus_volume(&mut self, bus: impl Into<String>, volume: f32) {
        self.commands.push(AudioCommand::SetBusVolume {
            bus: bus.into(),
            volume,
        });
    }

    pub fn fade_music_to(&mut self, volume: f32, duration_seconds: f32) {
        self.commands.push(AudioCommand::FadeMusicTo {
            volume,
            duration_seconds,
        });
    }
}

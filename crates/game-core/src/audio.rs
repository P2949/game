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
        });
    }
}

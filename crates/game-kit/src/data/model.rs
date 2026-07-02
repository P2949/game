use super::{
    AudioFile, BeginnerActionFile, BeginnerAssetsFile, BeginnerControlsFile, BeginnerGameFile,
    BeginnerMapFile, BeginnerPrefabFile, BeginnerRuleFile, CustomRuleFile, SceneFlowFile,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AuthoringGameFile {
    pub(crate) version: u32,
    pub(crate) assets: BeginnerAssetsFile,
    pub(crate) controls: BeginnerControlsFile,
    pub(crate) prefabs: Vec<BeginnerPrefabFile>,
    pub(crate) maps: Vec<BeginnerMapFile>,
    pub(crate) scene_flow: Option<SceneFlowFile>,
    pub(crate) audio: AudioFile,
    pub(crate) actions: Vec<BeginnerActionFile>,
    pub(crate) custom_rules: Vec<CustomRuleFile>,
    pub(crate) rules: Vec<BeginnerRuleFile>,
}

impl From<BeginnerGameFile> for AuthoringGameFile {
    fn from(file: BeginnerGameFile) -> Self {
        Self {
            version: file.version,
            assets: file.assets,
            controls: file.controls,
            prefabs: file.prefabs,
            maps: file.maps,
            scene_flow: file.scene_flow,
            audio: file.audio,
            actions: file.actions,
            custom_rules: file.custom_rules,
            rules: file.rules,
        }
    }
}

impl From<&BeginnerGameFile> for AuthoringGameFile {
    fn from(file: &BeginnerGameFile) -> Self {
        file.clone().into()
    }
}

impl From<&AuthoringGameFile> for AuthoringGameFile {
    fn from(file: &AuthoringGameFile) -> Self {
        file.clone()
    }
}

impl From<AuthoringGameFile> for BeginnerGameFile {
    fn from(file: AuthoringGameFile) -> Self {
        Self {
            version: file.version,
            assets: file.assets,
            controls: file.controls,
            prefabs: file.prefabs,
            maps: file.maps,
            scene_flow: file.scene_flow,
            audio: file.audio,
            actions: file.actions,
            custom_rules: file.custom_rules,
            rules: file.rules,
        }
    }
}

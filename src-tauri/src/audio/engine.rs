use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

use kira::{
    sound::static_sound::{StaticSoundData, StaticSoundHandle},
    AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Tween,
};

#[derive(Clone)]
pub struct RpgMakerAudioCommand {
    pub kind: String,
    pub name: Option<String>,
    pub volume: Option<i32>,
    pub pitch: Option<i32>,
    pub pan: Option<i32>,
    pub position: Option<f64>,
    pub duration: Option<f64>,
}

pub struct KiraAudioEngine {
    www_dir: PathBuf,
    manager: AudioManager<DefaultBackend>,
    bgm: Option<StaticSoundHandle>,
    bgs: Option<StaticSoundHandle>,
    me: Option<StaticSoundHandle>,
    se: Vec<StaticSoundHandle>,
    decoded_sounds: HashMap<PathBuf, StaticSoundData>,
}

impl KiraAudioEngine {
    pub fn new(www_dir: PathBuf) -> Result<Self, String> {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(|error| error.to_string())?;

        Ok(Self {
            www_dir,
            manager,
            bgm: None,
            bgs: None,
            me: None,
            se: Vec::new(),
            decoded_sounds: HashMap::new(),
        })
    }

    pub fn play(&mut self, command: RpgMakerAudioCommand) -> Result<(), String> {
        match command.kind.as_str() {
            "bgm" => self.play_persistent(command, PersistentAudioKind::Bgm),
            "bgs" => self.play_persistent(command, PersistentAudioKind::Bgs),
            "me" => self.play_persistent(command, PersistentAudioKind::Me),
            "se" => self.play_se(command),
            other => Err(format!("unsupported RPG Maker audio kind: {other}")),
        }
    }

    pub fn stop(&mut self, kind: &str) -> Result<(), String> {
        match kind {
            "bgm" => self.stop_handle(PersistentAudioKind::Bgm, Tween::default()),
            "bgs" => self.stop_handle(PersistentAudioKind::Bgs, Tween::default()),
            "me" => self.stop_handle(PersistentAudioKind::Me, Tween::default()),
            "se" => {
                for mut handle in self.se.drain(..) {
                    handle.stop(Tween::default());
                }
                Ok(())
            }
            "all" => {
                self.stop("me")?;
                self.stop("bgm")?;
                self.stop("bgs")?;
                self.stop("se")
            }
            other => Err(format!("unsupported RPG Maker audio kind: {other}")),
        }
    }

    pub fn fade_out(&mut self, command: RpgMakerAudioCommand) -> Result<(), String> {
        let tween = tween_from_seconds(command.duration.unwrap_or_default());
        match command.kind.as_str() {
            "bgm" => self.stop_handle(PersistentAudioKind::Bgm, tween),
            "bgs" => self.stop_handle(PersistentAudioKind::Bgs, tween),
            "me" => self.stop_handle(PersistentAudioKind::Me, tween),
            other => Err(format!("unsupported RPG Maker fade target: {other}")),
        }
    }

    fn play_persistent(
        &mut self,
        command: RpgMakerAudioCommand,
        kind: PersistentAudioKind,
    ) -> Result<(), String> {
        self.stop_handle(kind, Tween::default())?;

        let Some(name) = command.name.as_deref().filter(|name| !name.is_empty()) else {
            return Ok(());
        };
        let path = self.audio_path(kind.folder(), name)?;
        let sound = self.sound_data(&path, &command, kind.loops())?;
        let handle = self
            .manager
            .play(sound)
            .map_err(|error| format!("failed to play {}: {error}", path.display()))?;

        *self.handle_slot(kind) = Some(handle);
        Ok(())
    }

    fn play_se(&mut self, command: RpgMakerAudioCommand) -> Result<(), String> {
        let Some(name) = command.name.as_deref().filter(|name| !name.is_empty()) else {
            return Ok(());
        };
        let path = self.audio_path("se", name)?;
        let sound = self.sound_data(&path, &command, false)?;
        let handle = self
            .manager
            .play(sound)
            .map_err(|error| format!("failed to play {}: {error}", path.display()))?;

        self.se.push(handle);
        self.retain_live_se();
        Ok(())
    }

    fn stop_handle(&mut self, kind: PersistentAudioKind, tween: Tween) -> Result<(), String> {
        if let Some(mut handle) = self.handle_slot(kind).take() {
            handle.stop(tween);
        }
        Ok(())
    }

    fn handle_slot(&mut self, kind: PersistentAudioKind) -> &mut Option<StaticSoundHandle> {
        match kind {
            PersistentAudioKind::Bgm => &mut self.bgm,
            PersistentAudioKind::Bgs => &mut self.bgs,
            PersistentAudioKind::Me => &mut self.me,
        }
    }

    fn retain_live_se(&mut self) {
        self.se
            .retain(|handle| handle.state() != kira::sound::PlaybackState::Stopped);
    }

    fn sound_data(
        &mut self,
        path: &Path,
        command: &RpgMakerAudioCommand,
        loops: bool,
    ) -> Result<StaticSoundData, String> {
        let mut sound = self
            .decoded_sound(path)?
            .volume(rpg_maker_volume_to_decibels(command.volume.unwrap_or(100)))
            .playback_rate(rpg_maker_pitch_to_rate(command.pitch.unwrap_or(100)))
            .panning(rpg_maker_pan_to_panning(command.pan.unwrap_or(0)));

        if command.position.unwrap_or_default() > 0.0 {
            sound = sound.start_position(command.position.unwrap_or_default());
        }
        if loops {
            sound = sound.loop_region(..);
        }

        Ok(sound)
    }

    fn decoded_sound(&mut self, path: &Path) -> Result<StaticSoundData, String> {
        if let Some(sound) = self.decoded_sounds.get(path) {
            return Ok(sound.clone());
        }

        let sound = StaticSoundData::from_file(path)
            .map_err(|error| format!("failed to load {}: {error}", path.display()))?;
        self.decoded_sounds
            .insert(path.to_path_buf(), sound.clone());
        Ok(sound)
    }

    fn audio_path(&self, folder: &str, name: &str) -> Result<PathBuf, String> {
        let encoded_name = name;
        for extension in ["ogg", "m4a"] {
            let path = self
                .www_dir
                .join("audio")
                .join(folder)
                .join(format!("{encoded_name}.{extension}"));
            if path.is_file() {
                return Ok(path);
            }
        }

        Err(format!(
            "missing RPG Maker audio file: audio/{folder}/{name}.ogg or .m4a"
        ))
    }
}

#[derive(Clone, Copy)]
enum PersistentAudioKind {
    Bgm,
    Bgs,
    Me,
}

impl PersistentAudioKind {
    fn folder(self) -> &'static str {
        match self {
            Self::Bgm => "bgm",
            Self::Bgs => "bgs",
            Self::Me => "me",
        }
    }

    fn loops(self) -> bool {
        matches!(self, Self::Bgm | Self::Bgs)
    }
}

fn rpg_maker_volume_to_decibels(volume: i32) -> Decibels {
    let gain = (volume.clamp(0, 100) as f64 / 100.0).max(0.000_001);
    Decibels((20.0 * gain.log10()) as f32)
}

fn rpg_maker_pitch_to_rate(pitch: i32) -> f64 {
    (pitch.max(1) as f64) / 100.0
}

fn rpg_maker_pan_to_panning(pan: i32) -> f32 {
    (pan.clamp(-100, 100) as f32) / 100.0
}

fn tween_from_seconds(seconds: f64) -> Tween {
    Tween {
        duration: Duration::from_secs_f64(seconds.max(0.0)),
        ..Default::default()
    }
}

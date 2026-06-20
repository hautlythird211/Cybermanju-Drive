use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub duration_secs: f64,
    pub width: u32,
    pub height: u32,
    pub codec: String,
    pub fps: f64,
    pub bitrate: u64,
    pub audio_codec: Option<String>,
    pub audio_sample_rate: Option<u32>,
    pub audio_channels: Option<u32>,
    pub container: String,
    pub total_frames: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Buffering,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackPosition {
    pub current_secs: f64,
    pub total_secs: f64,
    pub speed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    pub index: i32,
    pub track_type: String,
    pub codec: String,
    pub language: Option<String>,
    pub title: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPlayerState {
    pub state: PlaybackState,
    pub position: PlaybackPosition,
    pub volume: f64,
    pub is_muted: bool,
    pub is_fullscreen: bool,
    pub current_track: Option<TrackInfo>,
    pub available_tracks: Vec<TrackInfo>,
}

pub struct VideoEngine {
    state: Arc<Mutex<VideoPlayerState>>,
    command_tx: Option<mpsc::Sender<VideoCommand>>,
}

enum VideoCommand {
    Play(String),
    Pause,
    Resume,
    Stop,
    Seek(f64),
    SetVolume(f64),
    ToggleMute,
    SetSpeed(f64),
    ToggleFullscreen,
}

impl Default for VideoEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoEngine {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(VideoPlayerState {
            state: PlaybackState::Stopped,
            position: PlaybackPosition {
                current_secs: 0.0,
                total_secs: 0.0,
                speed: 1.0,
            },
            volume: 1.0,
            is_muted: false,
            is_fullscreen: false,
            current_track: None,
            available_tracks: Vec::new(),
        }));

        VideoEngine {
            state,
            command_tx: None,
        }
    }

    pub fn get_state(&self) -> VideoPlayerState {
        self.state.lock().unwrap().clone()
    }

    pub fn play(&mut self, _path: &Path) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.state = PlaybackState::Playing;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        if state.state == PlaybackState::Playing {
            state.state = PlaybackState::Paused;
        }
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        if state.state == PlaybackState::Paused {
            state.state = PlaybackState::Playing;
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.state = PlaybackState::Stopped;
        state.position.current_secs = 0.0;
        Ok(())
    }

    pub fn seek(&mut self, position_secs: f64) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.position.current_secs = position_secs.clamp(0.0, state.position.total_secs);
        Ok(())
    }

    pub fn set_volume(&mut self, volume: f64) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.volume = volume.clamp(0.0, 2.0);
        Ok(())
    }

    pub fn toggle_mute(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.is_muted = !state.is_muted;
        Ok(())
    }

    pub fn set_speed(&mut self, speed: f64) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.position.speed = speed.clamp(0.25, 4.0);
        Ok(())
    }
}

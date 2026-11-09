//! Replay Recorder & Serializer.
//!
//! Records all chaos events (key presses, releases, lifecycle actions) alongside
//! frame ticks so that crashes and anomalies can be 100% deterministically reproduced.

use super::anomaly_sentinel::AnomalyReport;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FuzzAction {
    Key { key: i32, pressed: bool },
    PauseApp,
    ResumeApp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub tick: usize,
    pub action: FuzzAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayLog {
    pub game_id: String,
    pub jar_path: String,
    pub seed: u64,
    pub total_ticks: usize,
    pub anomaly: Option<AnomalyReport>,
    pub events: Vec<ReplayEvent>,
}

#[derive(Debug, Clone)]
pub struct ReplayRecorder {
    pub game_id: String,
    pub jar_path: String,
    pub seed: u64,
    pub events: Vec<ReplayEvent>,
}

impl ReplayRecorder {
    pub fn new(game_id: String, jar_path: String, seed: u64) -> Self {
        Self {
            game_id,
            jar_path,
            seed,
            events: Vec::new(),
        }
    }

    pub fn record_key(&mut self, tick: usize, key: i32, pressed: bool) {
        self.events.push(ReplayEvent {
            tick,
            action: FuzzAction::Key { key, pressed },
        });
    }

    pub fn record_pause(&mut self, tick: usize) {
        self.events.push(ReplayEvent {
            tick,
            action: FuzzAction::PauseApp,
        });
    }

    pub fn record_resume(&mut self, tick: usize) {
        self.events.push(ReplayEvent {
            tick,
            action: FuzzAction::ResumeApp,
        });
    }

    pub fn save_crash_report(
        &self,
        anomaly: AnomalyReport,
        total_ticks: usize,
    ) -> Result<PathBuf, String> {
        let dir = Path::new("crashes");
        if !dir.exists() {
            fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create crashes directory: {}", e))?;
        }

        let filename = format!("{}_seed_{}.json", self.game_id, self.seed);
        let path = dir.join(filename);

        let log = ReplayLog {
            game_id: self.game_id.clone(),
            jar_path: self.jar_path.clone(),
            seed: self.seed,
            total_ticks,
            anomaly: Some(anomaly),
            events: self.events.clone(),
        };

        let json = serde_json::to_string_pretty(&log)
            .map_err(|e| format!("JSON serialization error: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Failed to write crash log: {}", e))?;

        Ok(path)
    }

    pub fn load_from_file(path: &Path) -> Result<ReplayLog, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read replay file {:?}: {}", path, e))?;
        let log: ReplayLog = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse replay JSON {:?}: {}", path, e))?;
        Ok(log)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_roundtrip_serialization() {
        let mut rec = ReplayRecorder::new("test_game".to_string(), "test.jar".to_string(), 42);
        rec.record_key(10, -5, true);
        rec.record_key(12, -5, false);
        rec.record_pause(20);
        rec.record_resume(30);

        let log = ReplayLog {
            game_id: rec.game_id.clone(),
            jar_path: rec.jar_path.clone(),
            seed: rec.seed,
            total_ticks: 35,
            anomaly: None,
            events: rec.events.clone(),
        };

        let json = serde_json::to_string(&log).unwrap();
        let parsed: ReplayLog = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.events.len(), 4);
        assert_eq!(parsed.seed, 42);
    }
}

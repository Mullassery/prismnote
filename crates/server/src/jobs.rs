use serde::{Deserialize, Serialize};

/// How a job is triggered.
/// - `manual`: only when "Run now" is pressed.
/// - `interval`: every `minutes`.
/// - `daily`: once per day at `time` ("HH:MM", 24h, server-local).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub kind: String,
    #[serde(default)]
    pub minutes: Option<u64>,
    #[serde(default)]
    pub time: Option<String>,
}

impl Default for Schedule {
    fn default() -> Self {
        Schedule { kind: "manual".into(), minutes: None, time: None }
    }
}

/// One execution of a job.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobRun {
    pub started_at: String,
    pub finished_at: String,
    pub status: String, // "success" | "failed"
    pub cells_ok: usize,
    pub cells_failed: usize,
    pub log: String,
}

/// A saved notebook (snapshot of its code cells) that can be run as a unit and,
/// optionally, on a schedule — like an Airflow DAG run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub cells: Vec<String>, // code cell sources, in order
    #[serde(default)]
    pub schedule: Schedule,
    pub created_at: String,
    #[serde(default)]
    pub last_run: Option<String>,
    #[serde(default)]
    pub last_status: Option<String>,
    #[serde(default)]
    pub runs: Vec<JobRun>, // most-recent-last, capped
}

pub fn jobs_path() -> String {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    format!("{}/.prismnote/jobs.json", home)
}

pub fn load_jobs() -> Vec<Job> {
    std::fs::read_to_string(jobs_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_jobs(jobs: &[Job]) {
    if let Ok(json) = serde_json::to_string_pretty(jobs) {
        let path = jobs_path();
        if let Some(dir) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, json);
    }
}

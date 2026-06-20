use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub schedule_id: String,
    pub notebook_id: String,
    pub cron_expression: String, // Standard cron format
    pub enabled: bool,
    pub created_at: String,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub retry_on_failure: bool,
    pub max_retries: u32,
    pub timeout_seconds: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionJob {
    pub job_id: String,
    pub schedule_id: String,
    pub notebook_id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub status: String, // "pending", "running", "success", "failed"
    pub error_message: Option<String>,
    pub execution_time_ms: Option<u64>,
    pub output_cells: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub notify_on_success: bool,
    pub notify_on_failure: bool,
    pub email_addresses: Vec<String>,
    pub webhook_url: Option<String>,
}

pub struct SchedulerManager {
    notebook_id: String,
    scheduler_dir: String,
}

impl SchedulerManager {
    pub fn new(notebook_id: String, base_dir: &str) -> Self {
        let scheduler_dir = format!("{}/.prismnote/scheduler", base_dir);
        let _ = fs::create_dir_all(&scheduler_dir);

        SchedulerManager {
            notebook_id,
            scheduler_dir,
        }
    }

    pub fn create_schedule(&self, cron_expression: &str) -> Result<ScheduleConfig> {
        // Validate cron expression
        self.validate_cron(cron_expression)?;

        let schedule_id = uuid::Uuid::new_v4().to_string();
        let schedule = ScheduleConfig {
            schedule_id: schedule_id.clone(),
            notebook_id: self.notebook_id.clone(),
            cron_expression: cron_expression.to_string(),
            enabled: true,
            created_at: chrono::Local::now().to_rfc3339(),
            last_run: None,
            next_run: self.calculate_next_run(cron_expression),
            retry_on_failure: false,
            max_retries: 3,
            timeout_seconds: 3600, // 1 hour default
        };

        self.save_schedule(&schedule)?;
        Ok(schedule)
    }

    pub fn update_schedule(
        &self,
        schedule_id: &str,
        cron_expression: Option<&str>,
        enabled: Option<bool>,
        retry_on_failure: Option<bool>,
        max_retries: Option<u32>,
        timeout_seconds: Option<u32>,
    ) -> Result<ScheduleConfig> {
        let mut schedule = self.load_schedule(schedule_id)?;

        if let Some(cron) = cron_expression {
            self.validate_cron(cron)?;
            schedule.cron_expression = cron.to_string();
            schedule.next_run = self.calculate_next_run(cron);
        }

        if let Some(e) = enabled {
            schedule.enabled = e;
        }
        if let Some(r) = retry_on_failure {
            schedule.retry_on_failure = r;
        }
        if let Some(mr) = max_retries {
            schedule.max_retries = mr;
        }
        if let Some(ts) = timeout_seconds {
            schedule.timeout_seconds = ts;
        }

        self.save_schedule(&schedule)?;
        Ok(schedule)
    }

    pub fn delete_schedule(&self, schedule_id: &str) -> Result<()> {
        let schedule_path = format!("{}/{}.schedule", self.scheduler_dir, schedule_id);
        fs::remove_file(&schedule_path)?;
        Ok(())
    }

    pub fn record_execution(
        &self,
        schedule_id: &str,
        status: &str,
        error_message: Option<&str>,
        execution_time_ms: Option<u64>,
    ) -> Result<ExecutionJob> {
        let job_id = uuid::Uuid::new_v4().to_string();
        let start_time = chrono::Local::now().to_rfc3339();

        let job = ExecutionJob {
            job_id: job_id.clone(),
            schedule_id: schedule_id.to_string(),
            notebook_id: self.notebook_id.clone(),
            start_time: start_time.clone(),
            end_time: Some(chrono::Local::now().to_rfc3339()),
            status: status.to_string(),
            error_message: error_message.map(|s| s.to_string()),
            execution_time_ms,
            output_cells: 0,
        };

        self.save_execution_job(&job)?;

        // Update schedule's last run
        let mut schedule = self.load_schedule(schedule_id)?;
        schedule.last_run = Some(start_time);
        schedule.next_run = self.calculate_next_run(&schedule.cron_expression);
        self.save_schedule(&schedule)?;

        Ok(job)
    }

    pub fn get_execution_history(&self, schedule_id: &str, limit: usize) -> Result<Vec<ExecutionJob>> {
        let jobs_dir = format!("{}/jobs", self.scheduler_dir);
        if !Path::new(&jobs_dir).exists() {
            return Ok(vec![]);
        }

        let mut jobs = vec![];
        if let Ok(entries) = fs::read_dir(&jobs_dir) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(job) = serde_json::from_str::<ExecutionJob>(&content) {
                        if job.schedule_id == schedule_id {
                            jobs.push(job);
                        }
                    }
                }
            }
        }

        // Sort by start_time descending (newest first)
        jobs.sort_by(|a, b| b.start_time.cmp(&a.start_time));
        jobs.truncate(limit);

        Ok(jobs)
    }

    pub fn list_schedules(&self) -> Result<Vec<ScheduleConfig>> {
        let mut schedules = vec![];
        if let Ok(entries) = fs::read_dir(&self.scheduler_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("schedule") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(schedule) = serde_json::from_str::<ScheduleConfig>(&content) {
                            if schedule.notebook_id == self.notebook_id {
                                schedules.push(schedule);
                            }
                        }
                    }
                }
            }
        }
        Ok(schedules)
    }

    pub fn get_schedule(&self, schedule_id: &str) -> Result<ScheduleConfig> {
        self.load_schedule(schedule_id)
    }

    pub fn set_notifications(
        &self,
        schedule_id: &str,
        config: NotificationConfig,
    ) -> Result<()> {
        let notif_path = format!("{}/{}.notif", self.scheduler_dir, schedule_id);
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&notif_path, content)?;
        Ok(())
    }

    pub fn get_notifications(&self, schedule_id: &str) -> Result<Option<NotificationConfig>> {
        let notif_path = format!("{}/{}.notif", self.scheduler_dir, schedule_id);
        if Path::new(&notif_path).exists() {
            let content = fs::read_to_string(&notif_path)?;
            Ok(Some(serde_json::from_str(&content)?))
        } else {
            Ok(None)
        }
    }

    fn validate_cron(&self, cron_expression: &str) -> Result<()> {
        let parts: Vec<&str> = cron_expression.split_whitespace().collect();
        if parts.len() != 5 {
            return Err(anyhow::anyhow!(
                "Invalid cron expression. Expected 5 fields: minute hour day month weekday"
            ));
        }

        // Basic validation (not full cron spec)
        for part in parts.iter() {
            if *part == "*" {
                continue;
            }
            // Could add more detailed validation here
            let _ = part.parse::<u32>();
        }

        Ok(())
    }

    fn calculate_next_run(&self, _cron_expression: &str) -> Option<String> {
        // Simplified: return current time + 1 day
        // In production, use a proper cron library like `crony`
        let next = chrono::Local::now() + chrono::Duration::days(1);
        Some(next.to_rfc3339())
    }

    fn save_schedule(&self, schedule: &ScheduleConfig) -> Result<()> {
        let schedule_path = format!("{}/{}.schedule", self.scheduler_dir, schedule.schedule_id);
        let content = serde_json::to_string_pretty(schedule)?;
        fs::write(&schedule_path, content)?;
        Ok(())
    }

    fn load_schedule(&self, schedule_id: &str) -> Result<ScheduleConfig> {
        let schedule_path = format!("{}/{}.schedule", self.scheduler_dir, schedule_id);
        let content = fs::read_to_string(&schedule_path)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn save_execution_job(&self, job: &ExecutionJob) -> Result<()> {
        let jobs_dir = format!("{}/jobs", self.scheduler_dir);
        fs::create_dir_all(&jobs_dir)?;

        let job_path = format!("{}/{}.job", jobs_dir, job.job_id);
        let content = serde_json::to_string_pretty(job)?;
        fs::write(&job_path, content)?;
        Ok(())
    }
}

//! Async WMI job handling with timeout support.
//!
//! Hyper-V WMI operations often return immediately with a job reference.
//! This module provides utilities for waiting on jobs with configurable
//! timeout and progress callbacks.

use crate::error::{Error, JobState, Result};
use crate::wmi::{WbemClassObjectExt, WmiConnection};
use std::time::{Duration, Instant};

/// Default polling interval for job status.
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Default job timeout.
pub const DEFAULT_JOB_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

/// Job progress information.
#[derive(Debug, Clone)]
pub struct JobProgress {
    /// Current job state.
    pub state: JobState,
    /// Percent complete (0-100).
    pub percent_complete: u32,
    /// Job status description.
    pub status: String,
    /// Elapsed time since job started.
    pub elapsed: Duration,
    /// Error code if job failed.
    pub error_code: Option<u32>,
    /// Error description if job failed.
    pub error_description: Option<String>,
}

impl JobProgress {
    /// Check if the job is still running.
    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }

    /// Check if the job completed successfully.
    pub fn is_completed(&self) -> bool {
        self.state.is_completed()
    }

    /// Check if the job failed.
    pub fn is_failed(&self) -> bool {
        self.state.is_failed()
    }
}

/// Configuration for job waiting.
#[derive(Debug, Clone)]
pub struct JobWaitConfig {
    /// Maximum time to wait for job completion.
    pub timeout: Duration,
    /// Polling interval for job status.
    pub poll_interval: Duration,
}

impl Default for JobWaitConfig {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_JOB_TIMEOUT,
            poll_interval: DEFAULT_POLL_INTERVAL,
        }
    }
}

impl JobWaitConfig {
    /// Create a new configuration with specified timeout.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Default::default()
        }
    }

    /// Set the polling interval.
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }
}

/// Job waiter for async WMI operations.
pub struct JobWaiter<'a> {
    connection: &'a WmiConnection,
    config: JobWaitConfig,
}

impl<'a> JobWaiter<'a> {
    /// Create a new job waiter.
    pub fn new(connection: &'a WmiConnection) -> Self {
        Self {
            connection,
            config: JobWaitConfig::default(),
        }
    }

    /// Create a job waiter with custom configuration.
    pub fn with_config(connection: &'a WmiConnection, config: JobWaitConfig) -> Self {
        Self { connection, config }
    }

    /// Create a job waiter with specified timeout.
    pub fn with_timeout(connection: &'a WmiConnection, timeout: Duration) -> Self {
        Self {
            connection,
            config: JobWaitConfig::with_timeout(timeout),
        }
    }

    /// Wait for a job to complete, returning the final progress.
    pub fn wait_for_job(&self, job_path: &str, operation: &'static str) -> Result<JobProgress> {
        self.wait_for_job_with_callback(job_path, operation, |_| {})
    }

    /// Wait for a job to complete with a progress callback.
    pub fn wait_for_job_with_callback<F>(
        &self,
        job_path: &str,
        operation: &'static str,
        mut callback: F,
    ) -> Result<JobProgress>
    where
        F: FnMut(&JobProgress),
    {
        let start = Instant::now();
        let mut last_progress: Option<JobProgress> = None;

        loop {
            let elapsed = start.elapsed();

            // Check timeout
            if elapsed > self.config.timeout {
                let last_state = last_progress
                    .as_ref()
                    .map(|p| p.state)
                    .unwrap_or(JobState::Unknown);
                let percent = last_progress.as_ref().map(|p| p.percent_complete);

                return Err(Error::job_timeout(
                    operation,
                    job_path,
                    self.config.timeout,
                    last_state,
                    percent,
                ));
            }

            // Query job status
            let progress = self.get_job_progress(job_path, elapsed)?;

            // Call progress callback
            callback(&progress);

            // Check if job is done
            if !progress.is_running() {
                if progress.is_completed() {
                    return Ok(progress);
                } else {
                    // Job failed
                    return Err(Error::job_failed(
                        operation,
                        progress.error_code.unwrap_or(0),
                        progress
                            .error_description
                            .clone()
                            .unwrap_or_else(|| "Unknown error".to_string()),
                        progress.state,
                    ));
                }
            }

            last_progress = Some(progress);

            // Sleep before next poll
            std::thread::sleep(self.config.poll_interval);
        }
    }

    /// Get current job progress without waiting.
    pub fn get_job_progress(&self, job_path: &str, elapsed: Duration) -> Result<JobProgress> {
        let job = self.connection.get_object(job_path)?;

        let state_val = job.get_u16("JobState")?.unwrap_or(0);
        let state = JobState::from(state_val);

        let percent_complete = job.get_u32("PercentComplete")?.unwrap_or(0);
        let status = job
            .get_string_prop("JobStatus")?
            .unwrap_or_else(|| "Unknown".to_string());

        let (error_code, error_description) = if state.is_failed() {
            let code = job.get_u32("ErrorCode")?;
            let desc = job.get_string_prop("ErrorDescription")?;
            (code, desc)
        } else {
            (None, None)
        };

        Ok(JobProgress {
            state,
            percent_complete,
            status,
            elapsed,
            error_code,
            error_description,
        })
    }
}

/// Wait for a WMI method result that may return a job.
///
/// Hyper-V WMI methods return:
/// - 0: Completed successfully (synchronous)
/// - 4096: Job started (check Job output parameter)
/// - Other: Error code
pub fn wait_for_method_result(
    connection: &WmiConnection,
    out_params: &windows::Win32::System::Wmi::IWbemClassObject,
    operation: &'static str,
    timeout: Duration,
) -> Result<()> {
    let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);

    match return_value {
        0 => Ok(()), // Completed synchronously
        4096 => {
            // Job started - need to wait
            let job_path = out_params
                .get_string_prop("Job")?
                .ok_or_else(|| Error::operation_failed(operation, 4096, "Job path not returned"))?;

            let waiter = JobWaiter::with_timeout(connection, timeout);
            waiter.wait_for_job(&job_path, operation)?;
            Ok(())
        }
        code => {
            // Get error description if available
            let error_desc = out_params
                .get_string_prop("ErrorDescription")?
                .unwrap_or_else(|| format!("Operation failed with code {}", code));

            Err(Error::operation_failed(operation, code, error_desc))
        }
    }
}

/// Wait for a WMI method result with progress callback.
pub fn wait_for_method_result_with_callback<F>(
    connection: &WmiConnection,
    out_params: &windows::Win32::System::Wmi::IWbemClassObject,
    operation: &'static str,
    timeout: Duration,
    callback: F,
) -> Result<()>
where
    F: FnMut(&JobProgress),
{
    let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);

    match return_value {
        0 => Ok(()),
        4096 => {
            let job_path = out_params
                .get_string_prop("Job")?
                .ok_or_else(|| Error::operation_failed(operation, 4096, "Job path not returned"))?;

            let waiter = JobWaiter::with_timeout(connection, timeout);
            waiter.wait_for_job_with_callback(&job_path, operation, callback)?;
            Ok(())
        }
        code => {
            let error_desc = out_params
                .get_string_prop("ErrorDescription")?
                .unwrap_or_else(|| format!("Operation failed with code {}", code));

            Err(Error::operation_failed(operation, code, error_desc))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_state_from_u16() {
        assert_eq!(JobState::from(2), JobState::New);
        assert_eq!(JobState::from(3), JobState::Starting);
        assert_eq!(JobState::from(4), JobState::Running);
        assert_eq!(JobState::from(5), JobState::Suspended);
        assert_eq!(JobState::from(6), JobState::ShuttingDown);
        assert_eq!(JobState::from(7), JobState::Completed);
        assert_eq!(JobState::from(8), JobState::Terminated);
        assert_eq!(JobState::from(9), JobState::Killed);
        assert_eq!(JobState::from(10), JobState::Exception);
        assert_eq!(JobState::from(11), JobState::Service);
        assert_eq!(JobState::from(0), JobState::Unknown);
        assert_eq!(JobState::from(99), JobState::Unknown);
        assert_eq!(JobState::from(1), JobState::Unknown);
    }

    #[test]
    fn test_job_state_predicates() {
        // is_running tests
        assert!(JobState::New.is_running());
        assert!(JobState::Starting.is_running());
        assert!(JobState::Running.is_running());
        assert!(JobState::Suspended.is_running());
        assert!(JobState::ShuttingDown.is_running());
        assert!(!JobState::Completed.is_running());
        assert!(!JobState::Terminated.is_running());
        assert!(!JobState::Killed.is_running());
        assert!(!JobState::Exception.is_running());
        assert!(!JobState::Service.is_running());
        assert!(!JobState::Unknown.is_running());

        // is_completed tests
        assert!(JobState::Completed.is_completed());
        assert!(!JobState::Running.is_completed());
        assert!(!JobState::Exception.is_completed());

        // is_failed tests
        assert!(JobState::Terminated.is_failed());
        assert!(JobState::Killed.is_failed());
        assert!(JobState::Exception.is_failed());
        assert!(!JobState::Completed.is_failed());
        assert!(!JobState::Running.is_failed());
        assert!(!JobState::Unknown.is_failed());
    }

    #[test]
    fn test_job_state_display() {
        assert_eq!(format!("{}", JobState::New), "New");
        assert_eq!(format!("{}", JobState::Running), "Running");
        assert_eq!(format!("{}", JobState::Completed), "Completed");
        assert_eq!(format!("{}", JobState::Exception), "Exception");
        assert_eq!(format!("{}", JobState::Unknown), "Unknown");
    }

    #[test]
    fn test_job_wait_config() {
        let config = JobWaitConfig::default();
        assert_eq!(config.timeout, DEFAULT_JOB_TIMEOUT);
        assert_eq!(config.poll_interval, DEFAULT_POLL_INTERVAL);

        let custom = JobWaitConfig::with_timeout(Duration::from_secs(60))
            .with_poll_interval(Duration::from_millis(50));
        assert_eq!(custom.timeout, Duration::from_secs(60));
        assert_eq!(custom.poll_interval, Duration::from_millis(50));
    }

    #[test]
    fn test_job_wait_config_defaults() {
        assert_eq!(DEFAULT_POLL_INTERVAL, Duration::from_millis(100));
        assert_eq!(DEFAULT_JOB_TIMEOUT, Duration::from_secs(300));
    }

    #[test]
    fn test_job_progress_predicates() {
        let running_progress = JobProgress {
            state: JobState::Running,
            percent_complete: 50,
            status: "In progress".to_string(),
            elapsed: Duration::from_secs(10),
            error_code: None,
            error_description: None,
        };
        assert!(running_progress.is_running());
        assert!(!running_progress.is_completed());
        assert!(!running_progress.is_failed());

        let completed_progress = JobProgress {
            state: JobState::Completed,
            percent_complete: 100,
            status: "Done".to_string(),
            elapsed: Duration::from_secs(30),
            error_code: None,
            error_description: None,
        };
        assert!(!completed_progress.is_running());
        assert!(completed_progress.is_completed());
        assert!(!completed_progress.is_failed());

        let failed_progress = JobProgress {
            state: JobState::Exception,
            percent_complete: 75,
            status: "Failed".to_string(),
            elapsed: Duration::from_secs(20),
            error_code: Some(123),
            error_description: Some("Something went wrong".to_string()),
        };
        assert!(!failed_progress.is_running());
        assert!(!failed_progress.is_completed());
        assert!(failed_progress.is_failed());
    }

    #[test]
    fn test_job_progress_clone() {
        let progress = JobProgress {
            state: JobState::Running,
            percent_complete: 50,
            status: "Working".to_string(),
            elapsed: Duration::from_secs(5),
            error_code: None,
            error_description: None,
        };

        let cloned = progress.clone();
        assert_eq!(cloned.state, progress.state);
        assert_eq!(cloned.percent_complete, progress.percent_complete);
        assert_eq!(cloned.status, progress.status);
    }
}

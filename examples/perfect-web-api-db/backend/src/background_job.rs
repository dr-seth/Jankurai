// Background-job certified-cell shell.
//
// Boundary contract:
// - Owns queue/job value objects, retry policy, visibility/claim decisions,
//   port traits, and application-level enqueue/claim/finish orchestration.
// - Does not own provider queue clients, cron/scheduler providers, raw payload
//   material, SQL execution, environment variables, or wall-clock reads.
//
// Production split recommendation:
// - Move pure value objects to domain/background_job.rs.
// - Move command functions and port traits to application/background_job.rs.
// - Keep queue providers, SQL, cron triggers, and webhook dispatch in adapters.

use crate::domain::{Account, AccountId, AuditOutcome, DomainError, OrganizationId};
use std::fmt;

// ---------------------------------------------------------------------------
// Background-job value objects
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JobId(pub String);

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PayloadRef(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobKind {
    EmailDelivery,
    WebhookDispatch,
    ReportExport,
    Custom(String),
}

impl JobKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::EmailDelivery => "email-delivery",
            Self::WebhookDispatch => "webhook-dispatch",
            Self::ReportExport => "report-export",
            Self::Custom(value) => value.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundJobRetryPolicy {
    pub max_attempts: u8,
    pub base_delay_seconds: u64,
    pub max_delay_seconds: u64,
}

impl BackgroundJobRetryPolicy {
    pub fn standard() -> Self {
        Self {
            max_attempts: 3,
            base_delay_seconds: 30,
            max_delay_seconds: 30 * 60,
        }
    }

    pub fn validate(&self) -> Result<(), BackgroundJobError> {
        if self.max_attempts == 0 {
            return Err(BackgroundJobError::InvalidPolicy {
                reason: "max_attempts must be greater than zero".to_string(),
            });
        }
        if self.base_delay_seconds == 0 {
            return Err(BackgroundJobError::InvalidPolicy {
                reason: "base_delay_seconds must be greater than zero".to_string(),
            });
        }
        if self.max_delay_seconds < self.base_delay_seconds {
            return Err(BackgroundJobError::InvalidPolicy {
                reason: "max_delay_seconds must be at least base_delay_seconds".to_string(),
            });
        }
        Ok(())
    }

    pub fn backoff_seconds_for_attempt(&self, attempt: u8) -> u64 {
        let exponent = attempt.saturating_sub(1).min(6) as u32;
        let multiplier = 2_u64.saturating_pow(exponent);
        self.base_delay_seconds
            .saturating_mul(multiplier)
            .min(self.max_delay_seconds)
    }
}

// ---------------------------------------------------------------------------
// Background-job aggregate
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundJob {
    pub id: JobId,
    pub queue: String,
    pub kind: JobKind,
    pub payload_ref: PayloadRef,
    pub created_by: AccountId,
    pub organization_id: Option<OrganizationId>,
    pub status: JobStatus,
    pub attempts: u8,
    pub max_attempts: u8,
    pub run_at_epoch_seconds: u64,
    pub locked_by: Option<String>,
    pub completed_at_epoch_seconds: Option<u64>,
    pub last_error: Option<String>,
}

impl BackgroundJob {
    pub fn new(
        id: impl Into<String>,
        queue: impl Into<String>,
        kind: JobKind,
        payload_ref: impl Into<String>,
        created_by: AccountId,
        organization_id: Option<OrganizationId>,
        run_at_epoch_seconds: u64,
        policy: &BackgroundJobRetryPolicy,
    ) -> Result<Self, BackgroundJobError> {
        policy.validate()?;

        let id = id.into();
        if id.trim().is_empty() {
            return Err(BackgroundJobError::EmptyJobId);
        }

        let queue = queue.into();
        if queue.trim().is_empty() {
            return Err(BackgroundJobError::EmptyQueue);
        }

        let payload_ref = payload_ref.into();
        if payload_ref.trim().is_empty() {
            return Err(BackgroundJobError::EmptyPayloadRef);
        }

        Ok(Self {
            id: JobId(id),
            queue,
            kind,
            payload_ref: PayloadRef(payload_ref),
            created_by,
            organization_id,
            status: JobStatus::Queued,
            attempts: 0,
            max_attempts: policy.max_attempts,
            run_at_epoch_seconds,
            locked_by: None,
            completed_at_epoch_seconds: None,
            last_error: None,
        })
    }

    pub fn is_available(&self, queue: &str, now_epoch_seconds: u64) -> bool {
        self.queue == queue
            && self.status == JobStatus::Queued
            && self.run_at_epoch_seconds <= now_epoch_seconds
            && self.locked_by.is_none()
    }

    pub fn mark_running(&mut self, worker_id: impl Into<String>) -> Result<(), BackgroundJobError> {
        let worker_id = worker_id.into();
        if worker_id.trim().is_empty() {
            return Err(BackgroundJobError::EmptyWorkerId);
        }
        if self.status != JobStatus::Queued || self.locked_by.is_some() {
            return Err(BackgroundJobError::JobNotAvailable {
                id: self.id.clone(),
            });
        }
        self.status = JobStatus::Running;
        self.locked_by = Some(worker_id);
        self.attempts = self.attempts.saturating_add(1);
        Ok(())
    }

    pub fn mark_succeeded(
        &mut self,
        now_epoch_seconds: u64,
    ) -> Result<(), BackgroundJobError> {
        if self.status != JobStatus::Running {
            return Err(BackgroundJobError::JobNotRunning {
                id: self.id.clone(),
            });
        }
        self.status = JobStatus::Completed;
        self.locked_by = None;
        self.completed_at_epoch_seconds = Some(now_epoch_seconds);
        self.last_error = None;
        Ok(())
    }

    pub fn mark_failed(
        &mut self,
        error: impl Into<String>,
        now_epoch_seconds: u64,
        policy: &BackgroundJobRetryPolicy,
    ) -> Result<BackgroundJobFailureDecision, BackgroundJobError> {
        policy.validate()?;
        if self.status != JobStatus::Running {
            return Err(BackgroundJobError::JobNotRunning {
                id: self.id.clone(),
            });
        }
        let error = error.into();
        self.last_error = Some(error);
        self.locked_by = None;

        if self.attempts >= self.max_attempts || self.attempts >= policy.max_attempts {
            self.status = JobStatus::Failed;
            self.completed_at_epoch_seconds = Some(now_epoch_seconds);
            Ok(BackgroundJobFailureDecision::Exhausted)
        } else {
            self.status = JobStatus::Queued;
            self.run_at_epoch_seconds = now_epoch_seconds
                .saturating_add(policy.backoff_seconds_for_attempt(self.attempts));
            Ok(BackgroundJobFailureDecision::RetryScheduled)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundJobFailureDecision {
    RetryScheduled,
    Exhausted,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundJobError {
    EmptyJobId,
    EmptyQueue,
    EmptyPayloadRef,
    EmptyWorkerId,
    AccountInactive,
    PermissionDenied { action: String },
    JobNotAvailable { id: JobId },
    JobNotRunning { id: JobId },
    InvalidPolicy { reason: String },
    AdapterFailure { reason: String },
}

impl fmt::Display for BackgroundJobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyJobId => f.write_str("background job id must not be empty"),
            Self::EmptyQueue => f.write_str("background job queue must not be empty"),
            Self::EmptyPayloadRef => f.write_str("background job payload_ref must not be empty"),
            Self::EmptyWorkerId => f.write_str("background job worker_id must not be empty"),
            Self::AccountInactive => f.write_str("inactive account cannot operate background jobs"),
            Self::PermissionDenied { action } => write!(f, "background job action denied: {action}"),
            Self::JobNotAvailable { id } => write!(f, "background job {id} is not available"),
            Self::JobNotRunning { id } => write!(f, "background job {id} is not running"),
            Self::InvalidPolicy { reason } => write!(f, "invalid background job policy: {reason}"),
            Self::AdapterFailure { reason } => write!(f, "background job adapter failed: {reason}"),
        }
    }
}

impl From<DomainError> for BackgroundJobError {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::AccountInactive => Self::AccountInactive,
            DomainError::PermissionDenied { action, .. } => Self::PermissionDenied { action },
            other => Self::AdapterFailure {
                reason: other.to_string(),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Port traits (adapter contracts)
// ---------------------------------------------------------------------------

pub trait BackgroundJobRepository {
    fn save_job(&self, job: &BackgroundJob) -> Result<(), String>;
    fn find_next_available(
        &self,
        queue: &str,
        now_epoch_seconds: u64,
    ) -> Result<Option<BackgroundJob>, String>;
}

pub trait BackgroundJobAuditLog {
    fn record_background_job_event(&self, event: BackgroundJobEvent) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// Audit event contract
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundJobEvent {
    pub actor_id: AccountId,
    pub job_id: JobId,
    pub action: BackgroundJobEventAction,
    pub outcome: AuditOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundJobEventAction {
    Enqueued,
    Claimed,
    Completed,
    Retried,
    Exhausted,
}

// ---------------------------------------------------------------------------
// Application commands
// ---------------------------------------------------------------------------

pub fn enqueue_job(
    actor: &Account,
    job_id: impl Into<String>,
    queue: impl Into<String>,
    kind: JobKind,
    payload_ref: impl Into<String>,
    organization_id: Option<OrganizationId>,
    run_at_epoch_seconds: u64,
    policy: &BackgroundJobRetryPolicy,
    jobs: &impl BackgroundJobRepository,
    audit_log: &impl BackgroundJobAuditLog,
) -> Result<BackgroundJob, BackgroundJobError> {
    actor.authorize("write")?;

    let job = BackgroundJob::new(
        job_id,
        queue,
        kind,
        payload_ref,
        actor.id.clone(),
        organization_id,
        run_at_epoch_seconds,
        policy,
    )?;

    jobs.save_job(&job)
        .map_err(|reason| BackgroundJobError::AdapterFailure { reason })?;

    let _ = audit_log.record_background_job_event(BackgroundJobEvent {
        actor_id: actor.id.clone(),
        job_id: job.id.clone(),
        action: BackgroundJobEventAction::Enqueued,
        outcome: AuditOutcome::Success,
    });

    Ok(job)
}

pub fn claim_next_job(
    actor: &Account,
    queue: &str,
    worker_id: impl Into<String>,
    now_epoch_seconds: u64,
    jobs: &impl BackgroundJobRepository,
    audit_log: &impl BackgroundJobAuditLog,
) -> Result<Option<BackgroundJob>, BackgroundJobError> {
    actor.authorize("view_admin")?;

    let Some(mut job) = jobs
        .find_next_available(queue, now_epoch_seconds)
        .map_err(|reason| BackgroundJobError::AdapterFailure { reason })?
    else {
        return Ok(None);
    };

    job.mark_running(worker_id)?;
    jobs.save_job(&job)
        .map_err(|reason| BackgroundJobError::AdapterFailure { reason })?;

    let _ = audit_log.record_background_job_event(BackgroundJobEvent {
        actor_id: actor.id.clone(),
        job_id: job.id.clone(),
        action: BackgroundJobEventAction::Claimed,
        outcome: AuditOutcome::Success,
    });

    Ok(Some(job))
}

pub fn complete_job(
    actor: &Account,
    mut job: BackgroundJob,
    now_epoch_seconds: u64,
    jobs: &impl BackgroundJobRepository,
    audit_log: &impl BackgroundJobAuditLog,
) -> Result<BackgroundJob, BackgroundJobError> {
    actor.authorize("view_admin")?;
    job.mark_succeeded(now_epoch_seconds)?;
    jobs.save_job(&job)
        .map_err(|reason| BackgroundJobError::AdapterFailure { reason })?;

    let _ = audit_log.record_background_job_event(BackgroundJobEvent {
        actor_id: actor.id.clone(),
        job_id: job.id.clone(),
        action: BackgroundJobEventAction::Completed,
        outcome: AuditOutcome::Success,
    });

    Ok(job)
}

pub fn fail_job(
    actor: &Account,
    mut job: BackgroundJob,
    error: impl Into<String>,
    now_epoch_seconds: u64,
    policy: &BackgroundJobRetryPolicy,
    jobs: &impl BackgroundJobRepository,
    audit_log: &impl BackgroundJobAuditLog,
) -> Result<BackgroundJob, BackgroundJobError> {
    actor.authorize("view_admin")?;
    let decision = job.mark_failed(error, now_epoch_seconds, policy)?;
    jobs.save_job(&job)
        .map_err(|reason| BackgroundJobError::AdapterFailure { reason })?;

    let action = match decision {
        BackgroundJobFailureDecision::RetryScheduled => BackgroundJobEventAction::Retried,
        BackgroundJobFailureDecision::Exhausted => BackgroundJobEventAction::Exhausted,
    };
    let outcome = AuditOutcome::Error(
        job.last_error
            .clone()
            .unwrap_or_else(|| "background job failed".to_string()),
    );
    let _ = audit_log.record_background_job_event(BackgroundJobEvent {
        actor_id: actor.id.clone(),
        job_id: job.id.clone(),
        action,
        outcome,
    });

    Ok(job)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Role;
    use std::cell::RefCell;

    struct FakeJobs {
        jobs: RefCell<Vec<BackgroundJob>>,
    }

    impl FakeJobs {
        fn new() -> Self {
            Self {
                jobs: RefCell::new(Vec::new()),
            }
        }
    }

    impl BackgroundJobRepository for FakeJobs {
        fn save_job(&self, job: &BackgroundJob) -> Result<(), String> {
            let mut jobs = self.jobs.borrow_mut();
            if let Some(existing) = jobs.iter_mut().find(|candidate| candidate.id == job.id) {
                *existing = job.clone();
            } else {
                jobs.push(job.clone());
            }
            Ok(())
        }

        fn find_next_available(
            &self,
            queue: &str,
            now_epoch_seconds: u64,
        ) -> Result<Option<BackgroundJob>, String> {
            Ok(self
                .jobs
                .borrow()
                .iter()
                .find(|job| job.is_available(queue, now_epoch_seconds))
                .cloned())
        }
    }

    struct FakeAuditLog {
        events: RefCell<Vec<BackgroundJobEvent>>,
    }

    impl FakeAuditLog {
        fn new() -> Self {
            Self {
                events: RefCell::new(Vec::new()),
            }
        }
    }

    impl BackgroundJobAuditLog for FakeAuditLog {
        fn record_background_job_event(&self, event: BackgroundJobEvent) -> Result<(), String> {
            self.events.borrow_mut().push(event);
            Ok(())
        }
    }

    fn policy() -> BackgroundJobRetryPolicy {
        BackgroundJobRetryPolicy {
            max_attempts: 2,
            base_delay_seconds: 10,
            max_delay_seconds: 60,
        }
    }

    #[test]
    fn enqueue_job_requires_write_authority() {
        let viewer = Account::new("v1", "viewer@example.com", Role::Viewer).unwrap();
        let jobs = FakeJobs::new();
        let audit = FakeAuditLog::new();

        let err = enqueue_job(
            &viewer,
            "job1",
            "email",
            JobKind::EmailDelivery,
            "payload://email/1",
            None,
            100,
            &policy(),
            &jobs,
            &audit,
        )
        .unwrap_err();

        assert!(matches!(err, BackgroundJobError::PermissionDenied { .. }));
    }

    #[test]
    fn enqueue_job_emits_audit_event_without_storing_payload_body() {
        let actor = Account::new("a1", "agent@example.com", Role::Member).unwrap();
        let jobs = FakeJobs::new();
        let audit = FakeAuditLog::new();

        let job = enqueue_job(
            &actor,
            "job1",
            "email",
            JobKind::EmailDelivery,
            "payload://email/1",
            None,
            100,
            &policy(),
            &jobs,
            &audit,
        )
        .unwrap();

        assert_eq!(job.payload_ref.0, "payload://email/1");
        assert_eq!(audit.events.borrow().len(), 1);
        assert_eq!(audit.events.borrow()[0].action, BackgroundJobEventAction::Enqueued);
    }

    #[test]
    fn claim_next_job_only_returns_due_queued_work() {
        let actor = Account::new("a1", "agent@example.com", Role::Member).unwrap();
        let worker = Account::new("w1", "worker@example.com", Role::Admin).unwrap();
        let jobs = FakeJobs::new();
        let audit = FakeAuditLog::new();

        enqueue_job(
            &actor,
            "job1",
            "email",
            JobKind::EmailDelivery,
            "payload://email/1",
            None,
            100,
            &policy(),
            &jobs,
            &audit,
        )
        .unwrap();

        assert!(claim_next_job(&worker, "email", "worker-1", 99, &jobs, &audit)
            .unwrap()
            .is_none());

        let claimed = claim_next_job(&worker, "email", "worker-1", 100, &jobs, &audit)
            .unwrap()
            .unwrap();
        assert_eq!(claimed.status, JobStatus::Running);
        assert_eq!(claimed.attempts, 1);
        assert_eq!(claimed.locked_by.as_deref(), Some("worker-1"));
    }

    #[test]
    fn failed_job_retries_then_exhausts() {
        let actor = Account::new("a1", "agent@example.com", Role::Member).unwrap();
        let worker = Account::new("w1", "worker@example.com", Role::Admin).unwrap();
        let jobs = FakeJobs::new();
        let audit = FakeAuditLog::new();
        let retry_policy = policy();

        enqueue_job(
            &actor,
            "job1",
            "email",
            JobKind::EmailDelivery,
            "payload://email/1",
            None,
            100,
            &retry_policy,
            &jobs,
            &audit,
        )
        .unwrap();

        let claimed = claim_next_job(&worker, "email", "worker-1", 100, &jobs, &audit)
            .unwrap()
            .unwrap();
        let retry = fail_job(
            &worker,
            claimed,
            "smtp unavailable",
            101,
            &retry_policy,
            &jobs,
            &audit,
        )
        .unwrap();
        assert_eq!(retry.status, JobStatus::Queued);
        assert_eq!(retry.attempts, 1);
        assert_eq!(retry.run_at_epoch_seconds, 111);

        let claimed_again = claim_next_job(&worker, "email", "worker-1", 111, &jobs, &audit)
            .unwrap()
            .unwrap();
        let exhausted = fail_job(
            &worker,
            claimed_again,
            "smtp still unavailable",
            112,
            &retry_policy,
            &jobs,
            &audit,
        )
        .unwrap();
        assert_eq!(exhausted.status, JobStatus::Failed);
        assert_eq!(exhausted.attempts, 2);
        assert_eq!(exhausted.completed_at_epoch_seconds, Some(112));
    }
}

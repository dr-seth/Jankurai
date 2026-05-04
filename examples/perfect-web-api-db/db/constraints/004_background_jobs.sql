-- Background-job certified-cell constraints.
--
-- This file is intentionally separate from the migration so migration analyzers
-- can reason about durable safety checks without executing provider-specific
-- queue adapters.

alter table background_jobs
  add constraint background_jobs_queue_not_empty
  check (length(trim(queue)) > 0);

alter table background_jobs
  add constraint background_jobs_payload_ref_not_empty
  check (length(trim(payload_ref)) > 0);

alter table background_jobs
  add constraint background_jobs_attempts_within_max
  check (attempts <= max_attempts);

alter table background_jobs
  add constraint background_jobs_running_requires_lock
  check (
    (status = 'running' and locked_by is not null)
    or (status <> 'running')
  );

alter table background_jobs
  add constraint background_jobs_terminal_requires_completion_time
  check (
    (status in ('completed', 'failed', 'cancelled') and completed_at_epoch_seconds is not null)
    or (status not in ('completed', 'failed', 'cancelled'))
  );

create index if not exists idx_background_jobs_claimable
  on background_jobs (queue, status, run_at_epoch_seconds)
  where status = 'queued' and locked_by is null;

create index if not exists idx_background_jobs_org_status
  on background_jobs (organization_id, status);

-- Background-job certified-cell durable queue shell.
-- Provider-specific queue backends remain adapters; this table captures the
-- minimum durable truth required for replayable proof, auditability, and safe
-- retry/exhaustion behavior.

create table if not exists background_jobs (
  id text primary key,
  organization_id text references organizations(id),
  queue text not null,
  kind text not null,
  payload_ref text not null,
  status text not null default 'queued' check (
    status in ('queued', 'running', 'completed', 'failed', 'cancelled')
  ),
  attempts integer not null default 0 check (attempts >= 0),
  max_attempts integer not null default 3 check (max_attempts > 0),
  run_at_epoch_seconds bigint not null check (run_at_epoch_seconds >= 0),
  locked_by text,
  created_by text not null references accounts(id),
  created_at_epoch_seconds bigint not null check (created_at_epoch_seconds >= 0),
  completed_at_epoch_seconds bigint,
  last_error text
);

create table if not exists background_job_events (
  id text primary key,
  job_id text not null references background_jobs(id),
  actor_id text not null references accounts(id),
  action text not null check (action in ('enqueued', 'claimed', 'completed', 'retried', 'exhausted')),
  outcome text not null check (outcome in ('success', 'denied', 'error')),
  recorded_at_epoch_seconds bigint not null check (recorded_at_epoch_seconds >= 0)
);

create index if not exists idx_background_job_events_job_id
  on background_job_events (job_id, recorded_at_epoch_seconds);

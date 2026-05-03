// Root library module for the perfect-web-api-db reference backend.
//
// Layer responsibilities (from Jankurai JANKURAI_STANDARD.md):
//
//   domain      — IDs, invariants, pure decisions. No IO.
//   application — commands, authorization, idempotency, transactions. No UI.
//   adapters    — DB, queue clients, external APIs, filesystem. No domain rules.

pub mod adapters;
pub mod application;
pub mod domain;

pub fn service_name() -> &'static str {
    "perfect-web-api-db"
}

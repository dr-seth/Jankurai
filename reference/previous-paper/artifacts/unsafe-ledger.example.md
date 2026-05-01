# Unsafe Ledger Example

## Boundary

- Owner: `crates/ffi/src/lib.rs`
- Unsafe site: `ffi_slice_from_raw_parts`
- Risk: raw pointer validity and lifetime projection

## Why Unsafe Is Needed

The C library returns a pointer/length pair. Rust cannot validate pointer provenance or lifetime without a checked wrapper.

## Invariants

- Pointer must be non-null when length is non-zero.
- Buffer must remain valid for the returned borrow lifetime.
- Buffer must not be mutated while borrowed.
- Length must be bounded by the allocation returned from the C library.

## Safe Wrapper

- Public callers use `ForeignBuffer::as_slice()`.
- Constructor validates null/length relationship and stores ownership token.

## Required Proof

- `cargo miri test -p ffi`
- sanitizer lane on CI for FFI fixtures
- fuzz corpus for malformed length values
- review by code owner before widening unsafe surface

## Removal Path

Replace the C dependency with a safe Rust crate or require the upstream C library to expose an owned-copy API.

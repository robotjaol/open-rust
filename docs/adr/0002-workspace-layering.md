# ADR-0002: Layered Cargo workspace

Status: Accepted (2026-11-17)

## Context
A single crate would mix UI, storage and platform code and make tests slow and coupling easy.

## Decision
Nine crates: app-errors, app-config, app-security, app-logging, app-storage, app-platform, app-core (headless application services), ui-components (design system), and apps/desktop (shell and pages). Dependencies point inward: desktop to ui-components and app-core, app-core to services, services to app-errors.

## Consequences
Positive: core is testable without a window; integration tests run in milliseconds. Negative: more manifests to maintain.

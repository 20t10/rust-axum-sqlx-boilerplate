# Axum Clean Architecture + DDD Boilerplate Design

**Date:** 2026-05-26

## Goal

Create a reusable Rust Axum boilerplate inspired by cookie-cutter project starters that is ready for new product work immediately, while preserving strong Clean Architecture and DDD boundaries.

The boilerplate should optimize for a modular monolith, production-oriented local development, and repeatable bounded-context scaffolding.

## Chosen Direction

This boilerplate will use a workspace-based modular monolith:

- one deployable Axum application
- multiple workspace crates for separation of concerns
- bounded contexts modeled as modules/crates
- Clean Architecture layering inside each bounded context
- explicit infrastructure seams for future growth without premature microservice complexity

## Architecture

### Top-Level Shape

The project will be a Cargo workspace with one runnable HTTP application and supporting crates.

Planned structure:

- `apps/api`
  - Axum entrypoint
  - router composition
  - middleware stack
  - dependency wiring
  - startup and shutdown lifecycle

- `crates/shared-kernel`
  - cross-context primitives
  - common IDs
  - pagination types
  - auth claims primitives
  - time/clock abstractions
  - shared error/result foundations that are safe to reuse

- `crates/building-blocks`
  - technical cross-cutting concerns
  - config loading
  - tracing/logging setup
  - database pool bootstrap
  - JWT utilities
  - app error mapping helpers
  - outbox and job boundary contracts

- `crates/modules/identity`
  - reference bounded context
  - user and auth domain
  - application use cases
  - infrastructure persistence
  - HTTP presentation

- `crates/modules/<future-module>`
  - repeated module pattern for future bounded contexts such as billing, catalog, or orders

### Internal Module Layout

Each bounded context follows the same internal structure:

- `domain`
  - entities
  - value objects
  - domain services
  - repository interfaces
  - domain errors

- `application`
  - commands
  - queries
  - handlers
  - DTOs
  - orchestration logic

- `infrastructure`
  - SQLx repository implementations
  - read-model queries
  - persistence mapping
  - external-service adapters

- `presentation`
  - Axum handlers
  - request/response contracts
  - route registration
  - transport validation

### Architectural Rules

- Domain must not depend on Axum, SQLx, environment config, or transport concerns.
- Application coordinates use cases through ports and domain abstractions.
- Infrastructure implements outward-facing technical details and inward-facing ports.
- Presentation stays thin and delegates business behavior to application handlers.
- The runtime stays one deployable service, but module boundaries are explicit enough to prevent architectural drift.

## Runtime And Infrastructure

### Core Stack

The boilerplate will include:

- `axum` for HTTP
- `tokio` for async runtime
- `sqlx` for explicit write-side persistence and read-model queries
- PostgreSQL as the default primary data store
- JWT access/refresh authentication
- `tracing` for structured logging
- typed environment-driven configuration
- Docker for local service dependencies
- OpenAPI generation for the HTTP surface
- CI baseline for format, lint, test, and build

### Persistence Strategy

The persistence model is hybrid in the sense that the architecture separates:

- explicit repositories and transactional application flows for writes
- focused read-model query services for reads

The boilerplate will still use `sqlx` as the persistence engine on both sides, but the design distinguishes write-side repositories from read-side query models so teams can evolve both independently.

Persistence rules:

- write-side behavior lives behind repository ports
- read-side endpoints can use dedicated query services/read models
- transactions are owned by application/infrastructure boundaries, never by HTTP handlers
- migrations are first-class and included from the start

### Auth Baseline

The starter includes:

- user module in `identity`
- register endpoint
- login endpoint
- refresh endpoint
- `me` endpoint
- route guards
- role-aware authorization hooks
- example protected route

Auth responsibilities:

- domain models auth-related business concepts
- application handles authentication use cases
- infrastructure owns JWT encoding/decoding and persistence concerns
- presentation applies guards and transport mappings

### Background Jobs

The boilerplate will include a minimal background-job seam rather than a heavy scheduler framework.

Planned direction:

- define outbox/job contracts in shared technical layers
- keep initial implementation minimal and extendable
- avoid overbuilding workers before a real use case exists

This keeps the template practical while preserving a clean upgrade path for asynchronous workflows.

## Data Flow

Standard request flow:

1. Axum handler receives request and validates transport-level input.
2. Handler calls an application command or query handler.
3. Application layer coordinates domain rules through repository and service ports.
4. Infrastructure adapters implement those ports using SQLx and PostgreSQL.
5. Application result is mapped into a standard API response or error envelope.

## Error Model

Error boundaries are explicit:

- domain errors represent business rule failures
- application errors represent use-case and policy failures
- infrastructure errors are translated before crossing inward
- presentation maps internal errors to consistent HTTP responses

Rules:

- SQL and driver details must not leak into domain/application APIs
- transport error responses should be standardized across modules
- business failures and technical failures should remain distinguishable

## Testing Strategy

The scaffold will include multiple testing layers:

- unit tests for domain entities, value objects, and policies
- application tests with mocked ports
- repository integration tests against PostgreSQL
- HTTP smoke tests for routing, auth guards, and error envelopes
- one end-to-end auth flow as a working reference

Testing goals:

- keep domain logic easy to test without infrastructure
- prove infrastructure adapters with realistic database tests
- ensure the starter demonstrates how new modules should be tested

## Starter Features Included

The initial template should ship with:

- health check
- typed config loading
- database bootstrap
- SQL migrations
- seed baseline
- identity module
- register/login/refresh/me endpoints
- example protected route
- role-aware guard example
- OpenAPI docs endpoint
- Docker local development setup
- CI baseline

## Developer Workflow

The project will expose a first-class `Makefile` as the main developer interface.

### Standard Commands

Planned commands:

- `make setup`
- `make dev`
- `make up`
- `make down`
- `make reset-db`
- `make migrate`
- `make seed`
- `make fmt`
- `make lint`
- `make test`
- `make test-unit`
- `make test-integration`
- `make check`
- `make docs`
- `make ci`

### Command Design Rules

- commands should be idempotent where practical
- command names should reflect developer intent
- CI should mirror local workflows as closely as possible
- the Makefile should orchestrate tools, not hide large opaque logic

## Scaffolding Commands

The boilerplate will include generator-style developer commands.

### Primary Generator

Canonical command:

- `make create-module name=billing`

### Behavior Of `create-module`

When invoked, the command should:

- create a new crate under `crates/modules/<module>`
- generate the standard DDD/Clean Architecture internal layout
- add starter files for ports, use cases, infrastructure wiring, and presentation routing
- auto-register the new module in the Cargo workspace
- auto-register the module in the API composition/router wiring
- add a minimal smoke-test baseline for the generated module

### Guardrails

- fail if the module already exists
- normalize names into Rust-friendly crate/module conventions
- generate thin but compilable code
- update only known registration points for deterministic behavior

### Generator Implementation Approach

- `Makefile` exposes the public command
- generation logic lives in a repo-local script or small Rust scaffolder
- templates are stored locally in the repository
- generation behavior should be testable

## Non-Goals

The first version of the boilerplate should avoid:

- premature microservice extraction
- heavy worker infrastructure without a proven use case
- ORM-heavy abstractions that weaken domain boundaries
- oversized generic abstractions before the first real module patterns are proven

## Success Criteria

The boilerplate is successful if a new project can:

- start from one command-guided local setup
- ship new business modules using a repeatable structure
- preserve clear domain/application/infrastructure/presentation boundaries
- use a real auth and database baseline immediately
- add new bounded contexts with `make create-module name=<module>`
- stay maintainable as a modular monolith without needing early re-architecture

# ADR 0002: Specify Integration Tests with Gherkin

- Status: Accepted
- Date: 2026-10-17

## Context

Integration tests verify observable behavior across application boundaries. Writing these scenarios directly as Rust tests mixes behavioral specifications with test infrastructure and makes the intended contracts less accessible to non-Rust readers.

## Decision

All integration tests must be specified as Gherkin feature files using Given/When/Then scenarios. Rust step definitions may provide the test setup, invoke the application, and assert observable outcomes, but the scenario and its behavior must be declared in Gherkin. Integration tests must not introduce additional behavior scenarios that exist only as Rust test functions.

Unit tests remain ordinary Rust tests. Shared fixtures and helper functions may be implemented in Rust when useful, provided they do not replace the Gherkin scenario specification.

## Alternatives considered

- Write integration tests directly in Rust: rejected because behavioral scenarios should be explicit and readable independently of their implementation.
- Use Gherkin only for selected integration tests: rejected because a mixed convention would make coverage and expectations inconsistent.

## Consequences

- Every integration behavior is discoverable in a `.feature` file.
- Rust remains responsible for glue code and test infrastructure, not the top-level integration scenario.
- Existing integration tests must be migrated to Gherkin when touched or as part of the relevant issue.

# 9. Architecture Decisions

Architecture decisions are recorded in separate ADR documents to keep rationale and consequences authoritative in one place.

- [ADR 0001: Manage Collection and Web Delivery in One Server Lifecycle](../adr/0001-server-lifecycle.md) — accepted. Defines startup collection, server-owned scheduling with randomized jitter, failure handling, and shutdown.
- [ADR 0002: Specify Integration Tests with Gherkin](../adr/0002-gherkin-integration-tests.md) — accepted. Requires integration behavior to be specified in Gherkin feature files.
- [ADR 0003: Centralize Dependency Versions in the Workspace](../adr/0003-workspace-dependency-versions.md) — accepted. Requires dependency versions to be declared in the root workspace manifest.
- [ADR 0004: Support Multiple Fuel Price Sources](../adr/0004-multiple-fuel-price-sources.md) — proposed. Defines a source-agnostic design, a per-source contract, and admission criteria for fuel price sources.

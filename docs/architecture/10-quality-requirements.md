# 10. Quality Requirements

## Reliability

- A failed initial collection prevents the application from entering its normal running state.
- Later collection failures do not take down web delivery.
- Individual request errors are handled without uncontrolled process termination.

## Usability

- Users can inspect current fuel prices and station price history through the web interface.

## Maintainability

- Responsibilities and runtime dependencies are explicit.
- Existing crates are retained unless evidence supports changing boundaries.
- Architecture decisions and system structure remain documented.

## Operability

- Build and run steps are reproducible.
- Runtime settings are explicit.
- Startup, collection failures, and shutdown are observable in logs.

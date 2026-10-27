# ADR 0004: Support Multiple Fuel Price Sources

- Status: Proposed
- Date: 2026-10-27

## Context

Prices are collected from one fuel station website. Its page switches fuel types through JavaScript, so a given URL cannot reliably address a fuel type, and the parser depends on a single site's markup. The application should not depend on one website, and the fuel type and update time of every price must be interpreted consistently regardless of where the price came from.

Source selection today is implicit: the parser falls through to the mehr-tanken parser when a document lacks the legacy price list. This guesses the source from the document instead of stating it.

## Decision

### Source-agnostic design

The application is not tied to any fuel station website. A source is admitted by the criteria below, never by name. Websites named in this document are examples of the assessment, not recommendations, rankings, or a list of supported sources.

The existing mehr-tanken parser stays in place and is not removed.

### Source contract

Every source parser must satisfy the same contract, independent of the website:

- The fuel type is verified in the downloaded document. A mismatch rejects the whole document.
- A price carries all three decimals. Digits are never assumed or defaulted.
- An update time is resolved to an absolute instant in the configured source time zone. An entry whose time cannot be resolved is rejected instead of guessed.
- An entry without a price is skipped and counted as rejected. It is never stored as a price.

### Source selection and structure

The source is derived from the host of the configured URL. Document content never selects the parser. A URL with an unsupported host is rejected before anything is downloaded. Each source has its own parser that produces the common parsed result. Dispatch is a plain enumeration with member functions. A trait or plugin registry is not introduced until it is needed.

### Admission criteria

Required:

1. The fuel type is selectable without JavaScript, through the URL.
2. The document states its fuel type in a machine-readable way.
3. The sub-cent digit of the price is present in the markup.
4. The update time is absolute (date and time). Relative words such as "yesterday" are not sufficient. A missing year may be inferred from the fetch time only within a bounded window.
5. Each station has a stable identifier.
6. Name and address are present and can be separated into street and postal code with city.
7. Prices are in the delivered HTML as text, without obfuscation or JavaScript rendering.
8. Entries without a price are distinguishable from priced entries.

Preferred:

9. Coordinates are assignable to a station by identifier, not by list position.
10. The complete result list is on one page, with the search radius addressable through the URL.
11. The origin of the prices is stated.
12. The time zone of the update time is declared.
13. The terms of use permit automated retrieval.

### Example assessment

The saved pages in `examples/` were assessed against the criteria as of October 2026. Website layouts change, so this is a snapshot of examples.

| # | Criterion | mehr-tanken | clever-tanken | ich-tanke |
|---|-----------|-------------|---------------|-----------|
| 1 | Fuel type without JavaScript | No | Yes, query parameter (inferred, untested) | Yes, path segment |
| 2 | Fuel type stated in document | Yes, heading | Yes, URL and side box | Yes, title, heading, and URL |
| 3 | Sub-cent digit in markup | Yes | Yes | Yes |
| 4 | Absolute update time | Date without year | No, relative day and time | Yes, full date in an attribute |
| 5 | Stable station identifier | Yes, slug suffix | Yes, numeric | Yes, 32-digit hex |
| 6 | Name and address separable | Yes | Yes | Yes |
| 7 | Prices as delivered text | Yes | Yes | Yes |
| 8 | No-price marker | Not observed | Yes | Yes |
| 9 | Coordinates by identifier | Yes | No, by list position | Yes |
| 10 | Complete list on one page | No, load-more button | Yes | Yes |
| 11 | Price origin stated | Per-entry symbol, mixed origins | Per entry | Yes |
| 12 | Time zone declared | No | No | No |
| 13 | Terms permit retrieval | Not assessed | Not assessed | Not assessed |

Station names are brands in all three examples. The three examples also publish prices sourced from the same market transparency data, so several sources protect against layout changes and site outages, not against an outage of that data.

Not verified from saved pages: terms of use, live behavior such as bot protection and rate limits, the time zone of the update times (assumed to be the configured source time zone), and the mapping of source station identifiers to those of other sources.

### Station identity

Stations are stored by name and address. Different sources spell the same station differently (brand casing, "Str." versus "Straße", house number ranges). Before a second source writes to the shared database, a decision on station normalization or cross-source mapping is required.

## Alternatives considered

- Keep a single source: rejected because the application depends on one website's markup and cannot address fuel types reliably.
- Detect the source from the document: rejected because it guesses, and a layout change could route a document to the wrong parser.
- Common parser trait with a registry: rejected as premature abstraction.
- Replace mehr-tanken: rejected because no explicit decision to do so exists.

## Consequences

- A new source is accepted by checking the criteria, which keeps the choice independent of any website.
- The source contract makes fuel type and update time consistent across sources.
- The mehr-tanken parser stores a fixed sub-cent digit instead of the one in its markup, which violates the contract. It is corrected as the first step of the implementation.
- Cross-source station identity remains open and blocks writing a second source into the shared database.

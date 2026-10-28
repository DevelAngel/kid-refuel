Feature: Collect fuel prices from a source page
  The collector should accept valid price observations and report
  unusable pages as errors without aborting on single bad entries.

  Scenario: Reject a page for a different fuel type than configured
    Given the Mehr-Tanken Dresden E5 page is the input
    And the source URL is 'https://www.mehr-tanken.de/'
    When the collector dry-runs the page
    Then the collector fails
    And the log contains 'FuelTypeMismatch { expected: "E10", actual: "E5" }'

  Scenario: Reject a page without a fuel type heading
    Given the Mehr-Tanken Dresden E5 page is the input
    And its fuel type heading does not name a fuel type
    And the source URL is 'https://www.mehr-tanken.de/'
    When the collector dry-runs the page
    Then the collector fails
    And the log contains 'MissingFuelType'
    And the log contains 'Aktuelle Kraftstoff Preisübersicht'

  Scenario: Accept a page for the configured fuel type
    Given the Mehr-Tanken Dresden E5 page is the input
    And the source URL is 'https://www.mehr-tanken.de/'
    And the configured fuel type is 'e5'
    When the collector dry-runs the page
    Then the collector succeeds
    And the log contains 'prices fetched:'

  Scenario: Parse a valid price observation from the simulator
    Given the simulator page is the input
    When the collector dry-runs the page
    Then the collector succeeds
    And the log contains 'prices fetched: 8'
    And the log contains 'MyESSO'
    And the log contains 'Marienfelder Chaussee 171'

  Scenario: Skip malformed entries without aborting collection
    Given the page with malformed entries is the input
    When the collector dry-runs the page
    Then the collector succeeds
    And the log contains 'prices fetched: 1'
    And the log contains 'Valid Station'
    And the log does not contain 'Invalid Price Station'
    And the log does not contain 'Invalid Timestamp Station'

  Scenario: Report a missing price list without panicking
    Given the page without a price list is the input
    When the collector dry-runs the page
    Then the collector fails
    And the log does not contain 'panicked at'
    And the log contains 'MissingPriceList'

  Scenario Outline: Derive the simulator source from a loopback host
    Given the simulator page is the input
    And the source URL is '<url>'
    When the collector dry-runs the page
    Then the collector succeeds
    And the log contains 'prices fetched: 8'

    Examples:
      | url                    |
      | http://localhost:8080/ |
      | http://127.0.0.1:8080/ |
      | http://[::1]:8080/     |

  Scenario Outline: Derive the Mehr-Tanken source from the host
    Given the Mehr-Tanken Dresden E5 page is the input
    And the source URL is '<url>'
    And the configured fuel type is 'e5'
    When the collector dry-runs the page
    Then the collector succeeds
    And the log contains 'JET'

    Examples:
      | url                            |
      | https://mehr-tanken.de/        |
      | https://www.mehr-tanken.de/    |
      | https://tanken.mehr-tanken.de/ |

  Scenario Outline: Reject an unsupported host before downloading
    Given the source URL is '<url>'
    When the collector dry-runs the source URL
    Then the collector fails
    And the log contains 'UnsupportedSource'

    Examples:
      | url                                 |
      | https://example.com/                |
      | https://evilmehr-tanken.de/         |
      | https://mehr-tanken.de.example.com/ |

  Scenario: Reject a Mehr-Tanken page for a loopback URL
    Given the Mehr-Tanken Dresden E5 page is the input
    And the source URL is 'http://localhost:8080/'
    And the configured fuel type is 'e5'
    When the collector dry-runs the page
    Then the collector fails
    And the log contains 'MissingPriceList'

  Scenario: Reject a simulator page for a Mehr-Tanken URL
    Given the simulator page is the input
    And the source URL is 'https://www.mehr-tanken.de/'
    When the collector dry-runs the page
    Then the collector fails
    And the log contains 'MissingPriceList'

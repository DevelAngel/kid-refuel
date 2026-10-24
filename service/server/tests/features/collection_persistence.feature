Feature: Persist collected fuel prices
  The collector should store observations from a supported source so that
  the application can query the current prices.

  Scenario: Collect simulator prices and persist current observations
    Given the simulator provides eight fuel stations
    And the database is initialized with the application schema
    When the real collector processes the simulator HTML
    Then eight current station prices are queryable
    And MyESSO has its expected address and price

  Scenario: Collect prices from the current Mehr-Tanken page
    Given the Mehr-Tanken Dresden page is the input
    And the database is initialized with the application schema
    When the real collector processes the simulator HTML
    Then current Mehr-Tanken prices are queryable
    And JET has its expected address and price

  Scenario: Collect station coordinates from the simulator
    Given the simulator provides eight fuel stations
    And the database is initialized with the application schema
    When the real collector processes the simulator HTML
    Then all eight stations have coordinates

  Scenario: Collect station coordinates from the current Mehr-Tanken page
    Given the Mehr-Tanken Dresden page is the input
    And the database is initialized with the application schema
    When the real collector processes the simulator HTML
    Then all stored stations have coordinates

  Scenario: Keep coordinates that are already stored
    Given the simulator provides eight fuel stations
    And the database is initialized with the application schema
    And MyESSO is stored with other coordinates
    When the real collector processes the simulator HTML
    Then MyESSO keeps its stored coordinates
    And the other seven stations have coordinates

  Scenario: Store prices when the source provides no coordinates
    Given the page without station coordinates is the input
    And the database is initialized with the application schema
    When the real collector processes the simulator HTML
    Then one current station price is queryable
    And no station has coordinates


  Scenario: Keep only the latest twenty collection diagnostics
    When twenty-one collection diagnostics are recorded
    Then the collection history contains only the latest twenty runs


  Scenario: Limit stored response excerpts to sixteen KiB
    When a long response body is stored as an excerpt
    Then the response excerpt is bounded and valid UTF-8

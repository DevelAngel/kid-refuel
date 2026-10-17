Feature: Persist collected fuel prices
  The collector should store observations from a supported source so that
  the application can query the current prices.

  Scenario: Collect simulator prices and persist current observations
    Given the simulator provides eight fuel stations
    And the database is initialized with the application schema
    When the real collector processes the simulator HTML
    Then eight current station prices are queryable
    And MyESSO has its expected address and price

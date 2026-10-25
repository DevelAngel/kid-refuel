Feature: Provide price histories for comparison
  The comparison should provide the price changes of the requested
  stations for at most the last five weeks.

  Background:
    Given the database is initialized with the application schema

  Scenario: Provide only the requested stations
    Given station "Alpha" changed its price to 1.799 3 days ago
    And station "Beta" changed its price to 1.819 3 days ago
    And station "Gamma" changed its price to 1.839 3 days ago
    When the comparison is loaded for "Alpha, Beta"
    Then the comparison contains only "Alpha, Beta"

  Scenario: Ignore unknown stations
    Given station "Alpha" changed its price to 1.799 3 days ago
    When the comparison is loaded for "Alpha, Nowhere"
    Then the comparison contains only "Alpha"

  Scenario: Cover at most the last five weeks
    Given station "Alpha" changed its price to 1.899 60 days ago
    And station "Alpha" changed its price to 1.859 40 days ago
    And station "Alpha" changed its price to 1.829 30 days ago
    And station "Alpha" changed its price to 1.799 3 days ago
    When the comparison is loaded for "Alpha"
    Then the comparison for "Alpha" has the prices "1.859, 1.829, 1.799"

  Scenario: Keep the price of a station unchanged for more than five weeks
    Given station "Beta" changed its price to 1.699 50 days ago
    When the comparison is loaded for "Beta"
    Then the comparison for "Beta" has the prices "1.699"

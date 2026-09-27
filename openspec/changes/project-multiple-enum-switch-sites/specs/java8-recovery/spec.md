## ADDED Requirements

### Requirement: Complete class source projects multiple proved enum switches in one method

When complete class-source recovery finds multiple Java 8 enum remap-array switch sites in one method and independently proves every site from its selected helper table and enum definition, the system SHALL project all sites from the same recovered method AST as one atomic publication. The system MUST preserve each site's selector, case mapping, evaluation order, null behavior and selected-arm effects. If any site's proof or grouped AST emission fails or stops, the complete method SHALL retain its original integer-table switches; the output MUST NOT contain a partially projected subset of that method's sites.

#### Scenario: Two independently mapped enum selectors

- **WHEN** one complete Java 8 method switches first on a `Count` enum and then on an `Animal` enum, and both helper-table mappings are independently proved
- **THEN** complete class source SHALL emit both enum selectors and their mapped enum labels, and the full source family SHALL compile with `javac --release 8` and match the original class under `java -Xverify:all` for every tested value and null selector

#### Scenario: One site in the method cannot be proved

- **WHEN** a method has multiple enum switch sites but one site's table or enum mapping is missing, ambiguous or otherwise unproved
- **THEN** the entire method SHALL retain integer-table selectors and integer case labels, and the site refusal SHALL remain visible in the proof report

#### Scenario: Grouped emission stops

- **WHEN** grouped AST matching, output budget or cancellation stops after some sites have been examined
- **THEN** the method SHALL publish no enum-label edits, and each candidate's proof or stop outcome SHALL remain auditable

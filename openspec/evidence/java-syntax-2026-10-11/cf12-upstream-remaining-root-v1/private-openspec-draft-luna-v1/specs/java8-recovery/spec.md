## ADDED Requirements

### Requirement: Proof-gated switch continuations

Java 8 recovery MUST emit a switch continuation only when the decoded switch shape, the complete canonical CFG edges, and physical block ownership prove a unique join and the complete paths to it or to accurate terminal leaves. A conservative refusal MUST remain when the proof cannot close. The implementation MUST reuse existing region, arm, terminal, and budget semantics; it MUST NOT introduce a public recovery API, a new Region kind, a general graph framework, or a guessed join based on block order.

#### Scenario: A proved switch is followed by its enclosing straight continuation

- **WHEN** a switch arm in the enclosing frame contains one fully proved `Region::Switch` ending at an internal join, followed by one nonempty straight tail that reaches the enclosing arm boundary
- **AND** the complete canonical incoming/outgoing rows prove that the switch join belongs only to its switch paths and the tail has no external entry, non-Normal edge, overlap, or unowned block
- **THEN** recovery emits the switch and then the tail once in the enclosing arm, keeps the internal join outside every case body, and preserves each statement's physical source origin
- **AND** any `SwitchBreak`, fallthrough, and outer branch retains its existing exact target and nearest-scope checks

#### Scenario: A switch continuation is not a straight bounded tail

- **WHEN** the switch join's following path branches, loops, enters another switch, escapes the enclosing boundary, has an external predecessor, contains a non-Normal canonical edge, or does not match its walk's newly visited block set
- **THEN** recovery refuses the enclosing structure without duplicating, dropping, or partially consuming blocks

#### Scenario: A direct case target is the unique shared continuation of a finite DAG

- **WHEN** immediate post-dominator discovery has no result and a unique decoded direct-target candidate is reached by complete finite acyclic arm paths, where each path ends at that candidate or at an accurately decoded physical Return/Throw terminal with no canonical outgoing rows
- **AND** every internal branch is a proven comparison, all canonical edges are Normal and closed within one arm, arm closures are disjoint, and candidate incoming ownership is limited to the dispatch and proved arms
- **THEN** recovery may structure the switch with that candidate as its join, places terminal returns/throws in their original arm paths, and emits the candidate continuation once outside all arms

#### Scenario: A shared-join candidate has ambiguous or incomplete ownership

- **WHEN** no candidate or multiple candidates pass, an arm contains a cycle, nested switch, unknown terminal, non-Normal edge, unproved case-entry crossing, external incoming edge, or overlap on a non-join block
- **THEN** the switch remains conservatively refused with the existing refusal/coverage contract; recovery MUST NOT select a candidate by BCI ordering or by normal-flow successors alone

#### Scenario: Budget or cancellation stops a continuation proof

- **WHEN** the existing request budget or cancellation stops a charged node, edge, predecessor, or candidate check
- **THEN** recovery returns the existing Stop outcome at the accurate switch dispatch location, preserves actual usage and Stop reason/dimension, and publishes no partial text or source map

## ADDED Requirements

### Requirement: Proven member construction survives a sole-discard statement position

The system SHALL retain a completed member-construction proof when the finished instance has one immediate category-1 pop consumer and all physical arguments satisfy the existing statement-position safe-argument gate. It SHALL present the already-proved qualified or implicit member expression as one expression statement, without degrading it solely because a member target is present.

#### Scenario: Selected member target and safe discarded instance

- **WHEN** class-level identity/capture selection and call-site member proof are complete, the instance has exactly one immediate discard, and argument/effect/handler checks pass
- **THEN** the complete generated class SHALL preserve constructor effects, order and exceptions under JVM replay, including qualified nested-allocation and implicit-this cases

#### Scenario: Membership or argument proof is insufficient

- **WHEN** the selected member proof is absent, a check result is retained, the sole-discard proof fails, or a physical argument is a checked copy, invocation, field read or computed value outside the existing safe-argument gate
- **THEN** the system SHALL retain the existing physical spelling or source-complete refusal and SHALL NOT infer member statement eligibility from a null-check tail alone

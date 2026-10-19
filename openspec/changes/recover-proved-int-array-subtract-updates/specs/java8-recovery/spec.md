## ADDED Requirements

### Requirement: Proved int array subtraction statements retain their exact operator

Java 8 recovery MUST present a non-returning int-array compound subtraction only when the existing same-run lvalue and operand-use proof closes, and MUST retain the bytecode operand order and exact subtraction operator.

#### Scenario: Exact non-returning subtraction updates one array element
- **WHEN** an int-array read and store share the proved array/index duplication and all unique consumers close on exact isub with old value on the left
- **THEN** recovery SHALL emit the equivalent subtraction assignment with each array/index/RHS evaluation once and in bytecode order

#### Scenario: Existing addition and returned-addition gates remain exact
- **WHEN** the existing addition or returned-addition form is analyzed beside the new statement form
- **THEN** recovery SHALL retain its existing operator and proof boundary and SHALL NOT admit a returned subtraction through that addition-only path

#### Scenario: An unapproved arithmetic or incomplete use proof refuses
- **WHEN** the opcode/operation pair is not approved or any lvalue, ordering or unique-use obligation is unproved
- **THEN** recovery MUST keep the explicit refusal rather than suppressing duplication or publishing a guessed assignment

#### Scenario: Proof interruption does not publish partial recovery
- **WHEN** the proof budget stops or the request is cancelled
- **THEN** recovery MUST preserve the actual stop outcome and MUST NOT publish a partial compound assignment

# Project multiple proved enum switch sites in one method

The class-source facade currently refuses enum-label projection whenever one method has more than one remap-array enum switch. That refusal is broader than the available evidence requires: each candidate already has its own helper/enum map proof and points into the same recovered AST. A minimal two-selector Java 8 fixture confirms that the complete source remains compilable and behaviorally correct, but leaves both labels as integers.

Extend only the class-source publication step so all candidates in a method are applied to one copy of its same-run AST and emitted together. Keep the method atomic: if any site's proof or emission fails, publish the original recovered method with all integer switches. This change must reuse existing proof results and bounds; it does not change helper-map, enum identity, direct-ordinal, or single-method recovery behavior.

## Scope

- Include two or more independently proved Java 8 enum remap-array switch sites in one complete class-source method.
- Retain all-or-nothing publication for the method and existing stop/budget behavior.
- Add a focused positive two-site fixture and a partial-proof refusal fixture.

## Out of scope

- Changes to cross-class helper or enum proof rules.
- Switch expressions, direct `ordinal()` Smali/Java 21 lowering, multiple inputs to one switch, or arbitrary AST reconstruction.
- Production changes to single-site projection, which already passes the DT-31 replay.

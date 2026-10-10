# Instance candidate replay verifier v5 review

V5 fixes the owner-digest mismatch exposed by the preserved v4 run. Each render iteration now reloads the original class bytes keyed by `(group, jdk_leg, class_name)` before validating that class's owner and BLAKE3 digest. The strict digest and accepted-baseline comparisons remain unchanged.

The v4-to-v5 diff confirms this binding inside the per-class loop. A syntax-only AST parse passed. V5 was not executed; v1-v4 and their run records remain unchanged.

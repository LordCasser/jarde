# Static-array regression replay V2 — preparation

V2 is an independent successor to V1; V1 remains unchanged. It fixes the physical-field comparison to compare the actual `item`, `annotations`, `markers`, and `type_annotations` facts while excluding `declaration`, which is assembled presentation text and legitimately changes when instance field `b` is promoted. Methods remain compared in full, including physical report text and source maps.

The exact generated class set is checked using complete paths relative to the fresh classes directory, including any package path, so another class with the same basename cannot satisfy the check. All other preflight pins and the eight literal/ordered × two-JDK × default/all candidate replay legs are retained.

Preparation only: no candidate or toolchain has run. Output, when root executes the script, is `static-array-regression-root-v2/`.

# Instance candidate replay verifier v4 review

V4 retains the v3 verifier checks and fixes the exact no-clinit `acceptance_policy` literal to match the frozen replay manifest. A direct diff against v3 confirms the source change at the baseline assertion; v3 itself did not contain that fix. The output path/schema are advanced to v4.

Static equality against `instance-candidate-replay-root-v1/manifest.json` and Python AST parsing passed. The verifier remains unexecuted; the failed v2 execution record and all earlier versions remain untouched. No CLI, Rust, JDK, product, or raw evidence was changed.

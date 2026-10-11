# v2 handoff

Collector and verifier source are the two replacement files in this directory; v2.patch records their exact replacement delta from v1. pins.json freezes all scripts and input evidence.

No repository file or build was changed or executed. Collector/verifier were source-inspected only; Python AST parsing and all replay commands remain for root acceptance under the no-CLI constraint.

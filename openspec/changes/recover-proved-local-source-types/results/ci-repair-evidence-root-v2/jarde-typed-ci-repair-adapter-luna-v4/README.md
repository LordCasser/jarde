# Typed CI repair adapter v4 (private draft)

This is a separate v4 copy of the v3 adapter. Capture and acceptance use v4-specific paths. It preserves the historical 17 product-source, 10 test-source, and 50 canonical-input closure, its source-derived historical test names, fixed-seed workspace parsing, and all CI job/step checks. The workspace report again retains the exact `library_header` line alongside the 337/0/0 library summary.

For the local repair record, Cargo's `Running tests/...` target headers are checked in stderr: there must be exactly one header for each of the three expected integration binaries and no missing or duplicate target. Test names, individual `running N tests` markers, outcomes, and exact summaries are parsed independently from stdout. The combined CI log parser still requires each target header in its combined log and preserves its previous header-to-output binding checks.

The repair gate keeps the same nine additional pin paths and the original closure. Seven fixture/meeting pins retain their fixed hashes. The two recently edited test-source hashes are derived from the explicitly supplied product commit's Git blobs, then checked against live files and the repair record; v4 does not freeze their current pre-final hashes.

This is a static draft. No Cargo, Git, JDK, or CLI command was run. The currently recorded repair execution still fails, so this verifier has not accepted it or any CI result. Python AST parsing is the only validation performed here.

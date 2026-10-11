# For-condition local workspace acceptance v2

This private candidate derives from `jarde-for-condition-local-workspace-accept-luna-v1.py`. It targets the future root-v4 workspace receipt and pins the exact root-v4 runner SHA frozen by the match agent. It still pins the committed conditional parser/base, guard, and scanner; checks 95 metadata-derived batches over 360 targets, 98 commands, 196 raw streams, source pins before/after/live, seed/environment and resource limits; and independently parses all named outcomes and summaries.

The required named regression outcomes now include the crossing-array F1 control test currently named `f2_recovers_and_the_unprotected_control_only_projects_its_two_for_headers` and the Unicode test renamed to `the_ascii_presentations_only_change_in_the_two_proved_f1_for_headers`. The required test source definitions and exactly-once `ok` outcomes are checked independently of aggregate batch success. The root-v3 receipt is intentionally failed and must not be passed as successful evidence.

Invocation after root-v4 completes:

```text
python3 accept_workspace.py
```

Optional `--workspace` and `--out` arguments remain available. This candidate has only had syntax/symbol-table static checks; it was not executed. No shared file or evidence was changed and no Git/Cargo/toolchain command was run.

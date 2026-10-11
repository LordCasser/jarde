# Constructor conditional product own-CI verifier candidate

Private deterministic verifier candidate only. It has not been run against the workspace, final build, CLI, or a CI capture. The only validation performed here was Python AST parsing of this verifier and extraction of the 12 required names from the pinned workspace runner AST. No Cargo, JDK, JADX, CLI, Git, GitHub, network, or candidate-verifier execution was used.

The verifier adapts the for-condition v4 acceptance flow to `recover-proved-constructor-conditional-values`. It reuses the old conditional CI parser only for its metadata, Cargo artifact/outcome, stream, and Git-blob parsing helpers. It does not import the old OpenSpec verifier as current change evidence or claim old OpenSpec references as the source closure. Current source closure is reconstructed from the live product sources, literal includes, fixture tree, and the same frozen evidence root included by the current workspace runner; closure paths and bytes must match the runner's before/after pins and the selected product commit's Git blobs.

Workspace checks use the current constructor-conditional runner schema and `source_closure_count_at_start`. Cargo metadata independently derives the full target inventory and the batch plan; the candidate expects those derived values to be 361 targets and 95 batches, then checks recorded values, command order, named outcomes, summary totals, cleanup, and source pins. The source closure count is not a copied constant: it is checked against the independently reconstructed live closure and recorded start count (the current receipt reports 4727).

The exact 12 test names are read with `ast.literal_eval` from `REQUIRED_NAMED_TESTS` in the SHA-pinned current workspace runner. The verifier then requires each name to pass once in its expected integration or `jarde-java` library test target. The older for-condition required tests are not used as substitutes for this set.

Final-build validation binds the current focused v6 receipt and its CLI bytes to the final clean build receipt and final CLI path. It checks the final five guarded commands, current full source pins against product Git and live bytes, successful target cleanup, the CLI digest/size/mode, and equality with the focused candidate. Old for-condition guard/scanner/template references recorded by the workspace runner are checked only as runner provenance; they are not treated as the current focused result.

CI validation takes `--product-commit`, `--run-id`, and the captured run directory as inputs. It checks the four complete successful jobs (stable/specification, MSRV, supply chain, fuzz), every recorded step, exact capture stream/archive bindings, and both workflow-seeded workspace runs. For each seed, every target and every named test outcome is compared with the local raw workspace observations; masked public labels may bind only to one same-status local name. Aggregate totals are reconstructed from observed per-target outcomes and are never prefilled.

After the parent has final workspace/build/CLI/capture receipts, the intended invocation is:

```text
python3 /private/tmp/jarde-constructor-conditional-own-ci-verifier-prep-luna-v1/verify-own-ci.py \
  --product-commit <40-hex-commit> --run-id <run-id> \
  --capture <captured-run-directory> \
  --workspace /private/tmp/jarde-constructor-conditional-workspace-root-v1 \
  --final-build /private/tmp/jarde-constructor-conditional-final-lint-build-root-v1/execution.json \
  --final-cli /private/tmp/jarde-constructor-conditional-final-cli-root-v1 \
  --out <new-acceptance-json-path>
```

No result is inferred from the currently running workspace receipt; the parent should invoke the verifier only after all required evidence is complete.

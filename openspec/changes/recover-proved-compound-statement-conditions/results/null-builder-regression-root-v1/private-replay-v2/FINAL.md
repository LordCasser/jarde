# Final CLI v2 NullThenBuilder replay adapter, v2

This is v1 with one verifier correction: JADX puts both compiled classes under `defpackage/`, so the expected class paths for that profile now include the package directory. The other profiles retain their root-level class paths. The collector and all remaining checks are byte-for-byte unchanged from v1.

The fixture BLAKE3 value comes from the frozen prior class-source JSON at `/private/tmp/jarde-null-builder-runtime-root-v1/0.stdout.raw`: its `class.class_bytes` and class identity records report digest `f24b5b45eb94733793e5ee6d7f52f9cfe710d89bdbd65140930822d38f84a77b` and length 629. The prior execution record pins this raw JSON.

After the final build evidence exists, run:

```sh
CLI_SHA256="$(python3 -c 'import json; print(json.load(open("/private/tmp/jarde-compound-condition-focused-and-cli-root-v2/execution.json"))["cli"]["sha256"])')"
python3 /private/tmp/jarde-null-builder-final-cli-v2-replay-luna-v2/collect.py \
  --cli /private/tmp/jarde-compound-condition-candidate-cli-root-v2 \
  --cli-sha256 "$CLI_SHA256" \
  --out /private/tmp/jarde-null-builder-final-cli-v2-replay
python3 /private/tmp/jarde-null-builder-final-cli-v2-replay-luna-v2/verify.py \
  --execution /private/tmp/jarde-null-builder-final-cli-v2-replay/execution.json \
  --candidate-build /private/tmp/jarde-compound-condition-focused-and-cli-root-v2/execution.json \
  --cli-sha256 "$CLI_SHA256" \
  --out /private/tmp/jarde-null-builder-final-cli-v2-replay/verification.json
```

The root reports the final CLI as SHA-256 `378c410083dccb6d7d0ac08dcd7c0bcc519b57e29dd4a5d073b314779c06fe3d`, mode 0555, 60,819,568 bytes. The actual CLI SHA remains supplied from build evidence at invocation. No replay result is asserted by this adapter, and no toolchain or CLI command was run while preparing v2.

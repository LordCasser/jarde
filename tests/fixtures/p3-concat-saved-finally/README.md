# CF-16: saved concat return under shared finally

Replay with a fresh CLI and the pinned JADX checkout:

```sh
CARGO_TARGET_DIR=/private/tmp/jarde-cf16-concat-agent-target cargo build -p jarde-cli --locked
tests/fixtures/p3-concat-saved-finally/replay.sh /private/tmp/jarde-cf16-concat-agent-target/debug/jarde-cli /private/tmp/jarde-cf16-concat-evidence/replay
```

The fixed original `FinallyOnce.class` is SHA-256 `3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2`; JADX is pinned at `2fb1b16386941660fda07e9017285aec40fcb37f`. The minimal complete class only omits unrelated methods. Its `handled` has exactly the original instruction BCI/opcode sequence and all three exception rows: `[4,21)→31 IllegalArgumentException`, `[4,21)→65 any`, `[31,55)→65 any`. The normal saved return is BCI 18/20/29/30, the catch's concat/save/return is BCI 32–54/63/64, and the static cleanup copies are BCI 21–26, 55–60, 66–71.

The fixed full class runs `normal:1`, `caught:arg:1`, `state:1`. Pinned JADX's full class runs `normal:2`, `caught:arg:1`, `state:1`, so its normal branch performs the cleanup twice. Jarde recovers fixed `handled` and `escaping`, but fixed `main()` remains an independent quoted gap; the minimal complete class isolates the target. On that class, original and Jarde both run `normal:1`, `caught:arg:1` under `java -Xverify:all`, while JADX runs `normal:2`, `caught:arg:1`.

The exceptional fixture changes only `new IllegalArgumentException` and its constructor owner to a subtype whose `getMessage()` throws one pre-existing `IllegalStateException`. Original and Jarde both report `true:java.lang.IllegalStateException:message:1`: the `true` confirms object identity. The verifier-valid `non-concat`, `extra-consumer`, and `range-shrunk` variants all run normally but keep `handled` quoted in Jarde. They respectively remove the concat certificate, add a second stack consumer of the tail value, and remove tail coverage from the catch-all row.

Tracked class SHA-256 values:

| Class | SHA-256 |
| --- | --- |
| Minimal `v8/FinallyOnce.class` | `49ae3d9438d923ac047110a268424cc0e855920a9a82976d48cf99ddf4d581a6` |
| Exceptional `exceptional/FinallyOnce.class` | `68de8da237fd726f65ca9529ce7fca115aca20da33e79fa1e853efda66183a01` |
| `negatives/non-concat.class` | `8ea397c7d88166ba755a5c4ecbabc3599d72d7e546e3c2618053d65e1cf77373` |
| `negatives/extra-consumer.class` | `3f0d341747cdedd9483613e154d5bded3ad8a3dcfb4c94f463e4a3a5bce71119` |
| `negatives/range-shrunk.class` | `de8940afc4a16c5c89b805b4cc0b1622eb4f3b842cbd7c514a6f6d27341130e6` |

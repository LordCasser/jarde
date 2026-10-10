# Unicode CI replay verifier v4 delta

V4 is a new file; v3 remains unchanged. It corrects only the inventory comparison that stopped root’s v3 run. The collector’s `origin_count` is the number of source-map segments with a non-null primary origin, while the verifier separately counts every primary and derived origin for identity/BCI validation. V4 preserves that full per-origin validation and reports the total origin count, but compares inventory `origin_count` against primary-bearing segment count. It also takes `method_index` directly from each JSON method’s `item.index`.

The result schema and output filename are versioned as v4. The verifier has not been executed; only Python syntax parsing is intended before root’s run. No collector, raw evidence, product, or toolchain was changed.

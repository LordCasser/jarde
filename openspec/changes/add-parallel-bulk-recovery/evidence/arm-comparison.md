# A–D arms on the frozen corpus (2026-09-21, one machine)

A and B are the **historical calling shape**: the frozen sweep harness (`/tmp/jarde-bench/sweep3`,
built at `8807fa5`) asks for one method per request, each with a fresh `Budget`; B adds a retaining
facts store (`--cache 2000:67108864`). C and D are this change's CLI (`export`, release build,
explicit artifact-tree scope with the container roots declared) at one and at two-to-six workers.
All four run the same artifacts and the same methods.

| artifact | A sweep, no store | B sweep, retaining | C bulk, jobs=1 | D bulk, jobs=2 | D bulk, jobs=4 | D bulk, jobs=6 |
| --- | --- | --- | --- | --- | --- | --- |
| bcprov (2,430 classes / 15,003 methods) | 44.77 s | 5.23 s | **4.41 s** | 4.44 s | 4.65 s | 4.88 s |
| S2-009 (7,200 classes / 57,180 methods) | 324.83 s | 187.35 s | **19.31 s** | 15.71 s | 16.90 s | 17.91 s |

A/B are single samples taken while another build was running on the machine; C/D are medians of ten
interleaved samples from an idle machine (`raw-timing-interleaved.jsonl`). The A/B cells are inflated
by that load and by the harness's own per-request bookkeeping; they are here as anchors of the old
calling shape, not as a controlled comparison.

What the rows support:

* **The redesign is where the time went.** Per-class preparation with the prepared input, the
  operation's own streaming delivery and one ledger take S2-009 from 187 s (the old shape with the
  store that made it fast) to 19.3 s at one worker — an order of magnitude, before any parallelism.
  On bcprov the old shape with a store was already close (5.23 s) and the new one is at parity or
  slightly better at one worker.
* **The store is what the old shape needed and the operation provides differently.** A→B is 8.6×
  (bcprov) and 1.7× (S2-009) in the old shape; the new operation reads each container once per walk
  because the caller's store is attached, and work that the old shape repeated per request is done
  once per class.
* **Parallelism adds nothing yet.** C→D moves S2-009 by 19% at two workers and then regresses;
  bcprov does not move at all. The attribution table (`cost-attribution.md`) localises that loss
  inside the operation, and locating it is the open item.

Not supported by these rows: any cross-tool claim (jadx numbers live elsewhere and measure a
different product), any statement about other machines, or any conclusion from a single sample.

# GC09 raw-64 candidate-v2 comparison

Candidate CLI `43e41e391250e53982e7d2f948ae016b486b5d62da2b6e76707576455ff77f68` compared against the freshly replayed accepted baseline CLI `3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70` over 16 families × 2 JDKs × debug/no-debug = 64 rows.

| Metric | Candidate | New mismatches vs baseline | Improvements |
|---|---:|---:|---:|
| source_header | 64/64 | 0 | 0 |
| compile | 64/64 | 0 | 0 |
| runtime | 64/64 | 0 | 0 |
| behavior | 64/64 | 0 | 0 |
| field_type | 60/64 | 0 | 0 |
| field_api | 60/64 | 0 | 0 |
| method_api | 52/64 | 0 | 0 |
| class_api | 60/64 | 0 | 0 |

The 64 candidate Jarde sources all compiled and passed `-Xverify:all` runtime probes. The only four nonzero command rows are expected JADX javac failures for `InstanceRawLocal` across the four JDK/debug combinations. The runner recorded 842 status rows and 710 actual subprocesses.

The candidate retains the baseline API mismatch set: field API `InstanceRawLocal` (4 rows), method API `TypedReceiver`, `ShadowMethodT`, and `MultiFormalRawParam` (12 rows), and class API `RawOwnerChild` (4 rows). Each set has zero new mismatches versus the accepted baseline.

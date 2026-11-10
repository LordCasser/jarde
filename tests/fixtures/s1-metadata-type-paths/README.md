# S1 metadata type-path fixture

`ascii.jar` is the only embedded class archive for this fixture family. It is a deterministic ZIP using stored entries and the fixed 1980-01-01 timestamp.

The archive is regenerated from the four UTF-8 Java source units and the `ascii-javac23-none` matrix entry by `openspec/changes/restructure-demand-driven-recovery/results/s1-metadata-type-paths-v1/tools/fixtures.py`. The source inputs, compiler/runtime logs, per-case input pins, and JVM8/JVM23 oracle output are preserved under that result directory. The archive contains compiled Family, Marker, and Subject classes; Driver is kept only in the oracle support directory.

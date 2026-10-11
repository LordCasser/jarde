# Controlled nested-loop exit baseline

Only capture v3 is accepted for task1.2 by the actual frozen root verifier v2 (exit0).
25 commands/50 streams: two JDK originals and four fresh JADX complete-source runs
match all ten threshold/bound/window rows. Old frozen Jarde CLI generates four
fallback complete classes; their javac exits1, no generated class or runtime.
Default/all text, exact method identities, maps and all physical BCI origins agree.
The constructor/testFor/runCounts instruction counts are 3/40/47 per JDK;
classfile decoder and javap, method indices/flags/Code envelope and UTF8 spans
were independently checked. BLAKE3 report identities were not independently hashed.

v1 is a preserved collector failure at mkdir, after18 actual commands; v2 is a
complete observed different variant with tmp++ inside its condition (normal6).
v3 restores upstream continuation placement, tmp++ after the condition (normal12).
It tests reachable earlyreturn with ticks13/1/4 and normal20, zero-J/zero-K ticks0.
Neither v1 nor v2 is feature acceptance or task1.2 acceptance.

All raw commands, outputs, source/class/JAR inputs and frozen collector versions
are retained. Guard 5GiB free/target1GiB polls every second; no Cargo executed.
The guard env_overrides is template metadata, not measured Cargo environment.
This baseline uses the previously accepted computed-for CLI, not a new product.
CF11 and the 71-unit ledger remain open; implementation and post-fix verification
are outstanding.

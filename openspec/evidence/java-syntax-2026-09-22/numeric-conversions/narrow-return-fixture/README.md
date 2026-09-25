# Narrow integer return fixture evidence

`source.java.txt` is the ordinary Java 8 source whose selected methods return `int`.
`patch_descriptors.py` changes only the method_info descriptors for the 12 selected direct,
local, field, and synchronized returns; `descriptor-patches.json` proves the 14 Code attributes
have the same SHA-256 before and after the patch. The frozen patched class is then executed with
`java -Xverify:all` using a runner compiled against the patched descriptors.

`run_audit.py` records the complete Engine output, original/Jarde/JADX javac stages, and the
runtime output. The current Jarde source fails at the field post/pre narrow return statements;
JADX fails at its unsupported field update temporaries. No generated source was edited.

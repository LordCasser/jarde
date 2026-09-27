# DT-12 implementation replay

The fixed replay was run with the Jarde CLI built from this worktree and the frozen JADX checkout at `2fb1b16386941660fda07e9017285aec40fcb37f`. See `fixed/toolchain.txt` for Java/Javac/Javap/JADX versions and `fixed/outputs/` for per-tool logs, source files, and class hashes.

For both `-g` and `-g:none`, the full Jarde source set compiled with `javac --release 8` and its runner passed `java -Xverify:all`. Original, JADX, and Jarde each printed `TIMES=*:6:demo.DoubleOperations$1` and `DIVIDE=/:2:demo.DoubleOperations$2`. The Rust negative controls additionally prove refusal for verifier-valid ordinal and bridge-forwarding mutations, non-ASCII and non-literal String arguments, and a three-constant group. Output-budget and pre-cancellation tests confirm no partial dual-constant projection is published.

The supported source-argument slice remains deliberately narrow: exactly two ordered anonymous constant bodies, one ASCII String literal per constant, one shared proved `(String,int,String)` enum constructor relation, pure synthetic forwarding, and (for the frozen abstract enum shape) one direct interface with one abstract method implemented by both bodies. Non-ASCII, other constructor shapes, and broader interface layouts remain refused.

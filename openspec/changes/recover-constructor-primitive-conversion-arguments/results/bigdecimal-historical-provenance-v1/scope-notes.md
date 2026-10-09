# Scope and chronology note

This directory preserves historical bytes only. Its construction did not run Java, javac, JADX, Jarde, Cargo, or Git. The two legs contain the corrected one-class BigDecimal case for Corretto 8 and OpenJDK 23, including original source/class/input jar, the full one-source JADX output, original and JADX compile/run streams, the initial wrong-entrypoint run, and the later package-aware runtime recheck. The filtered manifests preserve actual argv, cwd, exit, JDK/tool identity and parent hashes while omitting the unrelated CT family.

The first addendum-02 JADX run used entrypoint `Main` although the generated source declares `defpackage.Main`, and exited 1. That failed run remains in the archive. The later FQCN-aware recheck invokes `defpackage.Main`, exits 0 on both JDKs, and matches the original stdout/stderr hashes.

## Time scope correction

The phrase “Jarde 0/18 overall” in the earlier EM-18 baseline analysis describes the historical baseline candidate, not the current candidate. The current candidate’s older 18-leg matrix is 16/18; the expanded legacy set is 18/22. These denominators must remain distinct. The corrected BigDecimal pair remains a 0/2 behavior match in the current replay.

The current permanent `legacy-regressions-root-v1` records are referenced by their manifest SHA and retain the current jar and class hashes. This archive did not regenerate or alter those results.

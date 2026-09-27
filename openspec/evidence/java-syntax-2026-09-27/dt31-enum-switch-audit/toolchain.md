# Toolchain snapshot

- JDK/Javac: OpenJDK `23.0.1` / `javac 23.0.1`; every Java comparison used `javac --release 8 -g:none` and `java -Xverify:all`.
- JADX checkout: `/Users/lordcasser/workspace/testzone/jadx`, clean `master` worktree at `2fb1b16386941660fda07e9017285aec40fcb37f`; decompiler is the checkout's installed `jadx` launcher.
- Jarde executable: `/tmp/jarde-cli-dt25-accepted`, SHA-256 recorded in `replay-baseline/summary.json`.
- The audit reused the supplied accepted Jarde executable. It created no Cargo target and did not modify either the main checkout or the JADX checkout.
- Replay invocation: `replay.py --jarde /tmp/jarde-cli-dt25-accepted --jadx /Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx --out <new-empty-directory>`.

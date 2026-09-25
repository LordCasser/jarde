# Fixture notes

`tests/fixtures/p3-type-qualifier/` contains one permanent Java 8 class,
`TypeQualifierProbe.class`; `arg0.java`, `arg0_2.java`, `ShadowOther.java`, and
`TypeQualifierRunner.java` are source-only helper and runner inputs. The probe has a
same-class static-call control and methods that use both default-package owners for
static calls, static field reads, and static field writes. The runner resets both
static fields before every operation and catches each `Throwable`, so null and
non-null cases each produce an independent result instead of aborting the later
cases.

The frozen class is 479 bytes, Java major version 52, with six methods and six Code
attributes. Its SHA-256 is
`0b8257791ce1d3544a42e5df31cfe8b9eee9ea5804b59c678caffb41f416634a`. A fresh
`javac --release 8 -g:none` compile of all fixture sources is byte-for-byte equal to
the frozen class. The original JVM run uses `java -Xverify:all`; the audit also
records `javap`, the complete Jarde class-source text, and a full JADX comparison.

The audit is `openspec/evidence/java-syntax-2026-09-22/type-name-shadowing/fixture/run_audit.py`.
It records the input and ending SHA-256 of `target/debug/jarde-cli`. The earlier
handoff baseline was `7527b03abc1e487121d204672b52043ca5f3aa1148c6d65b136e36f2bbe1f4aa`;
the worker rebuilt the CLI during this audit, so the current input and ending hash
are both `feed5c377a439490efb12a2a46bdfa0e94c32dffd3a83bd1583b70fe89317350` and
the audit records them unchanged. The source-only method-reference qualifier,
debug `java` parameter, and real `java` field package-prefix probes are kept under
the evidence fixture's boundary directory and are not part of the permanent
positive class; each original and JADX run succeeds while the current Jarde text
stops at its expected javac binding failure.

root独立复编译并运行的结果保存于evidence的`fixture/root/`：主类hash与冻结bytes一致，原/JADX的8项相等；同一feed5c构建下jarde零引用、完整javac成功，但6项运行不一致。root保留了`root/inputs/`源码与class供该次重放。Rust测试尚未运行：root移除了对测试源格式的冗余contains断言，并修正会错误拒绝`arg0_3`等安全后缀的前缀断言；后续需要实际Cargo RED/green和统一语料冻结。

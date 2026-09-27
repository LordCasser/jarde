# EM-25 主线独立复核

主线纳入审计提交 `f088cb11` 后，root 用冻结的 `replay.py` 和另一份基线 CLI `/tmp/jarde-cli-accepted-dt31`（SHA-256 `8f1f0012324350e4fc65c7fef4b3e3835102e6d4fda00df280b466270c98e727`）独立生成 `/tmp/jarde-em25-root-replay`。固定 JADX HEAD、六份测试和四份实现文件的 SHA 校验通过。原/JADX/Jarde 的完整 `BoxingAudit` 源码均以 Java 8 重编、`-Xverify:all` 运行成功，九行 stdout 与提交证据逐字相同；三方源码 SHA-256 分别为 `73d36bf8f1528f4d54eb6880e917c8f8566f822133221d68e1f3b8dd46a6d15d`、`b46d40385a4a7f3a0ae3fa22ed4261cbfd3df0afc19c0c44057b4c8bd113d09a`、`24dfd32612c2239cfe5aff7ed0c7b768d3312c7fd3a92159db5fa00c788c16e0`，均与审计记录相同。本切片只证明源码形式差异，没有运行缺口。

另外核对 [JLS §5.1.7](https://docs.oracle.com/javase/specs/jls/se8/html/jls-5.html#jls-5.1.7) 和 Java 8 `Boolean`/`Integer`/`Character.valueOf` 文档后，未接受原先把六种 `valueOf` 一律简写的实现范围。JLS 的装箱身份保证只与布尔、小范围 int、ASCII char 三种类库保证形成闭合交集；固定 `javac` 目前对 Byte/Short/Long 跑出相同身份，不是跨编译器证明。已将 OpenSpec 改为三种精确范围，并要求另外三种显式调用保持。此处不标记 EM-25 全项追平，引用 cast、重载和其它拆箱仍按账本独立审计。

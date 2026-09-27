# CF-16 共用汇合点 finally 独立验收

root 将 `ceb194e1` 合入含 CF-18 修复的主分支后，重新执行 `replay-acceptance.sh`。固定 JADX checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`，原 `TestTryCatchFinally$TestCls.class` SHA-256 为 `24a23e9f96826d8a7ffe104ba31bfc0abb005141572920604f82e13b4b082807`。原类、固定 JADX 默认和 `--no-finally`、Jarde 的四份完整源码均以 Java 8 重编并经 `java -Xverify:all` 执行，正常、具名 catch 和 `check()` 同为 `normal=true,f=true`、`exception=true,f=true`、`check=passed`。Jarde 源码 SHA-256 为 `e5efd39b99dd9dcdeeb4816a68204e3d5891cd7ae17c528c89c38dd13a3c941a`，只含一处 `this.f = true;`，并在 `finally` 后返回 `this.f`；默认 JADX 也只含一处，`--no-finally` 保留三处。

root 从原 class 的 `javap` 独立列出 `test(Object)` 的 26 个物理指令 BCI 和三行异常表，并与集成测试逐项核对。测试确认每个 BCI 都有源码来源，四个 Canonical 块 `[0,18,31,39]` 各有唯一 Region owner；证书将三份当前实例布尔字段写入和两条正常跳转归到共享 `finally`，BCI 39 的字段读取/返回只由后续 Region 拥有。主分支测试中的预算与取消均保持文本和来源图原子回滚。

同一脚本实时构造并 verifier 验证三种近邻负例：一份清理值不同、异常行缩短、第二条正常边改指第一份清理。Jarde 对三者均拒绝共享 `finally`，未发布部分折叠。仅改 `exc(Object)` 的 `Error` 变体保留 `test(Object)` 的全部 BCI 和异常表；原/Jarde 均在 catch-all 路径重抛同一异常对象且 `f=true`。布尔最终值本身不证明赋值次数，唯一赋值结论来自物理证书和完整生成源码。旧共享调用型、静态字段型及单出口 finally 的测试也在主分支全套测试中通过。

主分支 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-shared-join-finally --strict`、`git diff --check` 均通过。专用 replay Cargo target 已由脚本清理。验收范围限于固定共用汇合点及受证近邻；`FinallyOnce.handled/escaping` 等保护形态仍是 CF-16 的已证差距。

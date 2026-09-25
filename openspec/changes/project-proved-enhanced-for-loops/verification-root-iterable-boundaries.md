# `Iterable` 任务 4.1 的 root 独立复核

root 从[冻结源码](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/remaining-4-1/original/RawIterableIterator.java)以 `javac --release 8 -g` 重编 `RawIterableIterator` 和 `CatchNextScope` 加各自 runner，得到 class SHA-256 `830848b4e2214ae583d5a9dbb6fc94701017baee91d84c8109413590ccf1bcb2`、`67929431981e8b16ceecab8f6285af328a175d87be4e6809a5c7128dc132657d`，均与冻结 class 相同。`java -Xverify:all` 的 raw 原 class 六行依次为 `raw=3`、两条正常转换 `6,touches=2`、两条坏元素转换 `ClassCastException,touches=0/1`、`nextFailure=IllegalStateException,touches=1`；catch 原 class 为 `nextHandler=caught:1`。

root 将冻结的 JADX `-g` raw 源、Jarde `-g` raw 源和 JADX `-g` catch 源分别同对应 runner 以 `javac --release 8 -g:none` 重编、`java -Xverify:all` 运行，三者编译/验证均通过；两份 raw 与原 class 六行相同，JADX catch 则为 `nextHandler=escaped:IllegalStateException`。JADX `-g:none` raw 源单独重编运行，坏元素的“先 cast 后 touches++”路径变成 `ClassCastException,touches=1`，而原 class 为 `0`。Jarde catch 冻结源码为 explanation-only，缺返回，不能将其误报为可执行对照；它是独立的受保护区域基底债务。复核临时输出在 `/tmp/jarde-iterable-root-verify/`，无 Cargo target。

与[调试表成对控制](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/jadx-iterable-for-claim/analysis.md)和[raw `Object` 投影的手工 Java 8 验证](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/analysis.md)合看，第一实现子集可限定为：容器表达式具有真实 `Iterable` 源类型、调用点直接指向 `java/lang/Iterable.iterator()`；iterator 只供同一循环的 `hasNext()`/`next()` 使用；`next()` 是每轮首个可观察动作，且其处理器集合与循环头一致；元素只直接进入一次绑定/转换，raw 类型用新的 `Object` 绑定并原位保留显式 cast。仅同名 `iterator()`、其它 owner、额外 `next` 消费、先前副作用和不同处理器集合继续保留 `while`/保守引用。此范围是足以验证的第一切片，不声称覆盖 List/Collection 或所有 JADX 语法。

各组 `-g`/`-g:none`、非 `Iterable` 和完整诊断见[剩余 4.1 证据](../../evidence/java-syntax-2026-09-24/iterable-raw-projection/remaining-4-1/analysis.md)。独立 Cargo target 已由证据子代理清理，当前 root 可用磁盘约 16 GiB。

# 冻结证据：任务 1.1 / 1.3

详细证据位于 [`enum-constant-specific-body`](../../evidence/java-syntax-2026-09-25/enum-constant-specific-body/analysis.md)，包含 Java 8 源码、原/JADX/Jarde 完整类与 runner 文本、`-g`/`-g:none` 的完整 `javap -v -c -p`、class/source SHA、原与 JADX `javac --release 8` 重编及 `java -Xverify:all` 输出、工具版本和临时目录重放脚本。正例涵盖两个带体常量 `Op`、带体与普通常量混合 `Mixed`、两个普通常量 `Plain`。

任务 1.3 已由字节码事实完成：三个 enum 源文件均未声明构造器，即零个源参数；普通 enum 的物理构造器仍有注入的 `(String,int)` 描述符。`Op`/`Mixed` 还含匿名 owner 特定的 synthetic 访问构造器，如 `(String,int,Op$1)`；子类构造器向它传纯 `null` marker，再由它调用主类私有 `(String,int)` 构造器。`Op` 主类另带 `ACC_ABSTRACT`，其 `apply(II)I` 是无 Code 的抽象声明；两个专属子类分别提供 Code 实现。两种 debug 编译下这些构造和 `<clinit>` BCI 形状相同。基础 `recover-proved-enum-constants` 的 2.2/2.3 投影门限仍为单个源级 int 参数、一个构造器、具体类及全 Code 成员；2.3 只增加 `Measure` 的静态后缀。因而零源参数、anonymous owner、访问桥和抽象方法均超出已证明的基础准入范围。

使用冻结 CLI `jarde-cli 0.1.0`（SHA-256 `ca04265a4f412d59c29d6bd4a26b7d9cb961f72ae13e77684831c0b9e57b5145`）记录的基线结果是：三个 enum 均继续以普通字段/物理成员输出，Jarde 类源码均未通过 Java 8 编译；`Op`、`Mixed` 仍暴露 owner 构造器。Root 另从三个源文件重编 `-g`/`-g:none`，以 2.3 后 CLI（SHA-256 `2a3664efd81b31cbbfe13837b4250f339ba43fd3754083da36cd4e7a380a9bca`）对包含全部兄弟 class 的 JAR 请求 `demo.Op`、`demo.Mixed`、`demo.Plain`：六份类文本均保留普通常量字段与物理成员，未投影常量体；字段/方法数分别为 `3/8`、`3/7`、`3/6`。

任务 1.1 已由 Root 从仓库外的临时复制目录独立重放并验收：

```sh
cp -R openspec/evidence/java-syntax-2026-09-25/enum-constant-specific-body /tmp/enum-body-evidence-copy-0925
cd /tmp/enum-body-evidence-copy-0925
python3 replay.py --jarde-cli /tmp/jarde-generic-accepted-cli
shasum -a 256 -c manifest.sha256
```

该命令重建 79 个哈希证据文件，manifest 全部通过，`summary.json` 与冻结文件逐字段相同。所有 class/JAR 只在 `TemporaryDirectory` 中存在。基础枚举 change 已由 Root [完整验收](../recover-proved-enum-constants/verification-root.md) 并勾完任务；上文六份最新 CLI 结果也证明当前计划不会单独内联未证匿名子类体，因此本变更任务 1.2 的前置与拒绝门已满足。`Op` 的新零参数/匿名 owner 投影仍未实现。

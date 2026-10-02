# 限定 super 默认调用恢复（recover-qualified-super-default-calls，2026-10-02 实现取证）

承接[巡查 README](../README.md)。本目录（`sd/`）是实现切片的取证与验收记录：证明选择取证（下文 §1）、变体前后（`results-sd/before|after/`）、恢复输出与三方对照（`results-sd/`）。

## 1. 证明选择取证（任务 1.1：失败环节与消费点）

先 SHA 核对固定 fixture（`fam.jar` 七个成员与巡查 `results/fixture-sha256.txt` 逐一相同），主线重放双形基线与巡查记录逐字一致（`F1$Diamond`/`F1$Reabstract$Impl` 整方法拒绝，文案为 build.rs:21191 的 `no selected proof of a legal source qualifier and unique default binding`）。

**通道分层**（主线 `83ed9a2e`）：

| 环节 | 位置 | 已有能力 |
| --- | --- | --- |
| 消费点 | `crates/jarde-java/src/build.rs:21179` | **已有"直接超接口"白名单雏形**：`special_receiver` 对 interface 引用先比对调用类 header `direct_interfaces`，再按 (owner, name, descriptor) 查 `interface_super_calls` 证明表；查不到才落 21191 拒绝。root 决策 1 的直接性判据在此已完备 |
| 证明选择 | `src/facade.rs` `prove_interface_super_calls`（14055 起） | **已在读快照内跨类事实**：候选收集（header 直接接口过滤）后，经 `resolve_class_source_dependency_read_raw` 按需读限定符接口 header 与成员表（A16 计费、停止即 None）；`unique_source_default` 在接口闭包上要求唯一极大声明且 public/非 abstract/非 static/非 bridge/synthetic/恰一个 Code |
| 失败环节 | `interface_source_type_accessible`（facade.rs:14042） | **判据过严，且过严点唯一**：首片策略 `!owner.contains(&b'$')` 把嵌套接口限定符整体排除（注释自述"Nested source names need InnerClasses evidence"）。F1 家族接口全为嵌套（`F1$A`/`F1$B`），证明在读事实之前就被该门拒绝 |

**结论**：既非"证明未尝试读快照事实"（读取链路完备且顶层接口正路径早已有 `SpecialProbe.defaultCall → DefaultProbe.super.value()` 端到端恢复），也非直接性/默认绑定判据过严——是嵌套名 `$` 排除这一条首片遗留门。类源呈现契约（class_source.rs:1138–1149）本就规定 `$` 是合法标识符字符、嵌套类按字节携带名平铺拼写（`implements F1$A`），限定符取同一拼写与之一致，无需 InnerClasses 改造。

**修改**：删除该 `$` 排除（可访问性判据 public-或-同包保留不变），一处改动；其余证明（含菱形双限定各自独立收集/证明）全部复用。

## 2. 变体与负例（任务 1.2，`variants/`，javac 23 `--release 8 -g:none`）

| 变体 | 形态 | 原类运行 | 变更前 Jarde | 变更后 Jarde |
| --- | --- | --- | --- | --- |
| `SDDiamond$Use` | 菱形+自身覆写混合（`A.super.name()+B.super.name()`、`A.super.name()+"!"`）+ 带参（`greet(7)`）/void（`log("hi")`）默认方法限定调用 | `AB`/`A!`/`log:hi`/`g7` | 四方法整拒绝（21191 文案） | 全恢复：`SDDiamond$A.super.name() + SDDiamond$B.super.name()` 等（`results-sd/after/`） |
| `pkg/SDPacked$Use` | 包内嵌套公共接口限定符 | `PI` | 拒绝（21191） | `pkg.SDPacked$I.super.name()`（包名点号拼写，与类头 `implements` 一致） |
| `SDIndirect$Use`（`patch-negatives.py` 接口槽 `SDA`→`SDM`） | 限定符仅经 extends 链间接 | **HotSpot 校验器自拒**：`VerifyError: Bad invokespecial instruction: interface method reference is in an indirect superinterface`（`-Xverify:all` 下不可加载） | 拒绝：`names SDIndirect$SDA without a matching direct supertype or private declaration` | **逐字相同**（before/after 字节一致） |
| `SDAbstract$Use`（`patch-negatives.py` 调用属主 `SDB`→`SDC`） | 直接接口成员表中目标 abstract | `-Xverify:all` 链接通过（调用抛 `AbstractMethodError`，故只链接不调用） | 拒绝：21191 文案 | **逐字相同** |

两个负例是 `javac --release 8` 拒绝编译的源形，故由合法邻居类做**单索引常量池补丁**得到（`patch-negatives.py`，只改 2 字节索引、无长度变化）；同型补丁也钉进仓内回归测试 `tests/qualified_super_default.rs`（对 `tests/fixtures/qualified-super-default/` 的冻结 class 做同判据补丁）。取证中发现并记录：**间接限定符形连 JVM 校验器都不接受**——直接性要求是平台级不变量，Jarde 的 header 直接性门与之一致（负例在 Jarde 侧仍独立拒绝，不依赖 JVM）。

仓内回归钉子（`tests/qualified_super_default.rs`，5 例）：嵌套菱形双限定、单限定+void+带参、包内点号拼写三正例，间接/abstract 两负例；无修复时三正例 FAIL、两负例 PASS（git stash 双腿实测），修复后 5/5 PASS。

## 3. 三方对照（任务 3.2）

腿：原 class / 固定 JADX dev（`--no-debug-info`）/ Jarde 重编（`class-source` 每类输出 → `javac --release 8 -g:none`）。运行全部 `java -Xverify:all`，输出 SHA 见 `results-sd/sha256.txt`。

| 路径 | 原类 | JADX dev | Jarde 重编 |
| --- | --- | --- | --- |
| F1（fam.jar 整家族重编运行） | `AB`/`I:A`（== o1.out） | **限定符丢失**：`super.name()+super.name()`，重编失败（`找不到符号 name()`），run 腿不可产出（`results-sd/F1.jadx.compile-error.txt`） | `AB`/`I:A`（== o1.out，逐字） |
| SDDiamond | `AB`/`A!`/`log:hi`/`g7` | 同上四路限定符全丢，编译失败 | 逐字一致 |
| SDIndirect / SDAbstract（合法邻居，未补丁） | `SA` / `SB` | 限定符丢失，编译失败 | 逐字一致 |
| pkg.SDPacked | `PI` | 限定符丢失，编译失败 | 逐字一致 |

JADX dev 对**所有**限定 super 默认调用一律去限定符转写（含 void/带参/包内形）——本切片按 handoff 约束记录其错误转写而不照搬。F2 对照类：恢复文本与巡查基线逐字节一致（`<clinit>` 尾部 `return;` 拼写为**既有**偏差，JLS 8.7 禁止初始化块 return，故 fam.jar 含 F2 的整 jar 重编本就不闭合；F1 家族七类重编运行闭合，见 §3 第一行）。该既有偏差不在本切片范围，建议独立登记。

## 4. 通道零回退（任务 2.2）

`results-sd/channel-diff/`：主线（stash 腿）与修复腿对 `SpecialProbe`（普通 `super.value()`、private 接收者、副作用实参、`DefaultProbe.super.value()` 既有正路径）、`F1`/`F1$A`/`F1$B`/`F1$Reabstract`/`F1$Reabstract$C`/`F2` 的 class-source 输出**逐字节相同**；仅 `F1$Diamond`/`F1$Reabstract$Impl` 两方法由拒绝翻正（diff 即恢复行）。

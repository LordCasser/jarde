# 实现与验证记录（2026-10-06，coder）

本文件记录 `preserve-monitor-exit-evaluation-order` 的实现落点、锚实测与门禁结果；全部判定来自实测命令
与本工作树代码。插桩定夺见 [instrumentation.md](instrumentation.md)。

## 1. 代码落点

| 文件 | 位置 | 内容 |
|---|---|---|
| `crates/jarde-java/src/guard.rs` | `pub struct InnerMonitor`（+`returns()` 访问器）、`fn monitor`、新 `fn expression_inside` | 内层 pair 自带 `returns: Option<u32>`；该值只在"被 return 消费值的定义指令落在**内层** body `(body_start, exit_load)` 内，且 `expression_inside` 证明该值整条表达式链也落在内层 body 内"时写入。不满足时字段保持 `None`，呈现与改动前逐字相同（不新拒、不换位） |
| 同上 | `fn expression_inside(facts, value, range)` | 与渲染器同链的值遍历：`Definition::Instruction` 必须落在 `range` 内（其已证构造 site 的 `owned`/`expression` 也要落内）；沿 operand-stack（`Slot::Stack`）向操作数展开；局部读=名字（停）、`Entry`=方法输入（允许）、`Phi`/`Caught`=未证控制流边界（答否）。每访问值 `facts.charge` |
| `crates/jarde-java/src/build.rs` | `struct NestedPairBraces`（+`returns`、+`closed_without_return`） | 由 plan 的 `InnerMonitor::returns()` 填充 |
| 同上 | Monitor arm 的 `(Some(pair), Some(tree))` 分支 | 走完 region 树后、**关闭 braces 之前**渲染 `guarded_return(return_bci)`（失败路径仍 `region_quote`+`fallback`，诊断文本不变），把语句交给 `close_nested_pair(inner_return)`；`close_nested_pair` 把它作为内层 body 的**最后一条语句**追加 |
| 同上 | 同上（arm 末尾） | plan 自带内层 return 时跳过 `body.push(statement)`（该 return 已写在内层 braces 内） |
| 同上 | `fn close_nested_pair(inner_return)` / `instruction()` 的走中闭合调用点 | 走中闭合（pair 的 span 在 body 内结束）若遇 return 形，记 `closed_without_return`，arm 随即以 `jre_guard_body` 引注该语句（fail-closed，对 javac return 形不可达——见 instrumentation §1） |
| `crates/jarde-reader/src/classfile.rs` | `repository_class_fixtures_validate_without_false_target_rejections` | fixture 人口新计数 `(591, 2615, 282, 1775, 8)`（+6 class / +26 body，两腿 NL/NL$Box/SR），按既有惯例加一行来源注释 |

## 2. 锚实测（命令与输出）

`CLI = target/debug/jarde-cli`；协议：`class-source --format text` → 剥离 `^[[:space:]]*//` → `javac` → `java -Xverify:all`。

| # | 锚 | 命令/输入 | 结果 |
|---|---|---|---|
| 1 | **NL 判别（本片目标）** | `CLI class-source --input nl.jar --class NL` → `NL.java` | 呈现为 `synchronized (NL.LOCK) { synchronized (NL.class) { return "n" + arg0; } }`（内层块非空、return 在内层 braces 内） |
| 2 | NL 编译/运行（javac 23.0.1 `--release 8`） | `javac --release 8 -d out23 NL.java` | exit 0（仅 `-Xlint:-options` 源/目标过时警告） |
| 3 | NL 运行 | `java -Xverify:all -cp out23 NL` | **`nY`** |
| 4 | NL 原 class 运行 | `java -Xverify:all -cp nl.jar NL` | `nY`（与 3 逐字相同） |
| 5 | **改动前呈现的负面读数** | 用改动前二进制渲染同一 `nl.jar` 得旧文本，同法编译运行 | **`nN`**（求值越过内层 `monitorexit`；判别成立） |
| 6 | NL 真 javac 8 腿 | `javac -d v8-javac8 NL.java SR.java` → 同法剥离编译运行 | 渲染文本与 javac 23 腿**逐字相同**；运行 `nY` |
| 7 | SR 单层零回退 | 改动前/后渲染成员级 diff | `retInside`、`localAcross`、`voidBody`、`main` **逐字节相同**；仅 `nestedLock` 变化（return 移入内层 braces） |
| 8 | SR `voidBody` 安全拒 | 剥离文本 `javac --release 8` | 编译失败于 `jarde_refused_body`（找不到符号）——fail-closed 保持 |
| 9 | SR 同形锚 | SR 渲染文本 | `nestedLock` = `synchronized (SR.LOCK) { synchronized (SR.class) { return "n" + arg0; } }`（两腿逐字相同） |
| 10 | 既有 nested 呈现（`GuardReturnEffects#nested`） | 成员级 diff | return 从外层 braces 移入内层 braces（`return local4;`）；`effect`（单层 + 独立调用）引注逐字不变 |

## 3. 全量 fixture 回归扫（`tests/fixtures/**` 全部 591 个 `.class`）

协议：改动前二进制（`git stash` 后重建）与改动后二进制各渲染一遍
（`class-source --policy single-class --format text`），逐文件比较退出码与呈现文本。

| 量 | 结果 |
|---|---|
| 退出码 | 591/591 相同（无新拒、无新回退） |
| 呈现文本 | **585/591 逐字节相同**；6 处不同 |
| 6 处不同的定性 | 4 处 = 本片新增 fixtures（NL/SR × 两腿；新增文件，改动前不存在）；1 处 = `tests/fixtures/preserve-guarded-return-expression/v8/GuardReturnEffects.class`（nested 成员 return 移入内层）；1 处 = 新增 fixtures 各 class 的呈现（同一批）；另计 `GuardReturnEffects` 一处即全部既有变更 |

**既有 fixture 中仅 `GuardReturnEffects#nested` 一处呈现变化**：其字节码 `iload 4`（BCI 31）在内层
`monitorexit`（BCI 34）之前，故该局部读按同一不变量移入内层——与本片目标同类，非回退面变化；该 fixture
自己的测试（`tests/p3_sync_return.rs`）不钉位置，实测仍绿。`proposal.md` 的"既有 monitor 巡查渲染逐字节
不变"因此按"单层形与既有拒绝/降级面不变"读：单层形逐字节不变、零新拒、零新降级（见上表）。

## 4. 门禁

| 门禁 | 命令 | 结果 |
|---|---|---|
| 全量测试 | `cargo test --workspace --tests --locked --no-fail-fast` | **305 个测试二进制 / 2997 passed / 0 failed / 47 ignored**；同环境同命令的改动前基线为 304 / 2994 / 0 / 46（差 = 本片新增 1 个测试目标 + 3 个测试 + 1 个 ignored），已知 `p4_plugins` 负载敏感 flake 两次均未出现 |
| 格式 | `cargo fmt --all -- --check` | exit 0（clean） |
| clippy | `.github/workflows/ci.yml` 46–76 逐字（`cargo clippy --workspace --all-targets --all-features --locked -- <30 项 A + -D warnings>`） | 见 §4.1 |
| OpenSpec | `openspec validate --all --strict` | **300 passed, 0 failed** |

### 4.1 clippy

`clippy-driver` clean、`-D warnings` 下无输出（exit 0）。

## 5. 对照测试入库

- `tests/preserve_monitor_exit_evaluation_order.rs`：3 个非 ignored（NL probe 内层 return 逐字锚 + BCI 锚
  5/10/11/31/33/34、SR 四成员文本锚 + content 面、`GuardReturnEffects#nested` 结构锚与 `effect` 拒绝锚）
  + 1 个 ignored（`cargo test -- --ignored`：剥离注释 → `javac --release 8` → `java -Xverify:all` 得 `nY`
  并与原 class 输出比较；SR 剥离文本必须在 `jarde_refused_body` 上编译失败）；
- `tests/fixtures/preserve-monitor-exit-evaluation-order/`：巡查原件源 + 双腿字节（v8 = javac 23.0.1
  `--release 8`，v8-javac8 = Corretto 1.8.0_432），命令与 SHA256 见该目录 README；
- `tests/fixtures/corpus-fingerprint.json`：新增 8 个文件的 blake3/字节数（ignored regenerator 生成，diff 仅新增）。

## 6. 提交

见本轮提交（`docs(change)` 前 `feat`/`test` 各一）。不 push。

## 7. 与预审计的偏差

1. **定夺为 (a)**（内层 `returns` + return 留内层 braces），非 (b) 合成 temp：两者都在既有证明结构内，
   (b) 除同一注入点外还要新造合成命名/类型/声明/账本四面，回退面更大（instrumentation §2）。
2. **预审计行号漂移**：`guard.rs` 的 `returns` 文档与 build.rs 的 return 分支位置已移动
   （`returns` 文档现 ~320，build.rs return 推入现 ~14682）；机制与预审计描述一致。
3. **既有 nested fixture 呈现变化 1 处**（§3），与"corpus 逐字节不变"的字面读法不同；按不变量与同类
   收敛读作预期变化，已在 §3 记录证据与理由，供 root 判定是否需要在 proposal 里显式收窄该句。

## Why

[recover-javac8-getclass-null-check-idiom](../recover-javac8-getclass-null-check-idiom/) 已交付"null-check 拼写谓词"（`facts.rs::is_discarded_null_check` + `NullCheckSpelling{RequireNonNull,GetClass}`），修复了**参数限定符形**（`outer.new Inner(…)`，`outer` 是既有局部/参数）在真 javac 8 上的恢复。但**分配限定符形**（`new Outer().new Inner(…)`，外层是新分配的实例）在真 javac 8 下**仍被拒**——root 以合并态二进制实测：`N1.main` 源码区 quotes=13、`new N1().new Inner(3)` 未折叠。这是 [DT-03](../../evidence/jadx-feature-inventory-2026-09-27/declarations-types.md) 的**残留缺口**，属真实 Java 8 覆盖缺口（`new Outer().new Inner()` 是常见形），非润色。

**根因（root 已实测钉死，见 [getClass 片 tasks 3.5 追加取证](../recover-javac8-getclass-null-check-idiom/tasks.md)）**：真 javac 8 在嵌套分配之后、成员构造器之前插入被丢弃的 null-check 三元组 `dup(23); invokevirtual Object.getClass(24); pop(27)`；javac 23 对分配限定符**不发射任何检查**（`20: invokespecial N1.<init>` 后直接 `23: iconst_3; 24: invokespecial N1$Inner.<init>`）。`init.rs` 的 `new@1` 实例读者门（`fn verify_new` 约 681-711）把该 `dup` 计为"实例的读者"，而 `renders_its_reads`（1331-1347）不接受 `Operation::Duplicate`（落 `_ => false`）→ `written.is_empty()` → 拒绝。实测诊断逐字匹配 `init.rs:694` 模板：`"the instance the allocation at BCI 16 builds is read only by instructions this build quotes (BCIs 23), so the construction has no place in the body"`，对外码 `jre_new_shape`。

## What Changes

- 让 `new@1` 的实例读者门认得**被丢弃的 null-check 三元组**（`dup; <discarded null check>; pop`）为该分配站点自身的一部分——正如既有逻辑已把构造器自己的 `dup` 排除在读者外（`init.rs:677-678` 文档："The constructor's own copy is not a reader here: it is one of the three instructions this site owns, and `outside_readers` leaves the site's own instructions out"）。
- 复用 [getClass 片](../recover-javac8-getclass-null-check-idiom/) 已交付的 `facts.rs::is_discarded_null_check` 谓词识别该检查的两种拼写（`requireNonNull` / `getClass`），**不新增第二套拼写判据**。
- 分配限定符形恢复后，`new Outer().new Inner(…)` 呈现为源级限定语法；javac 9+（无检查）产物**逐字节零回退**；参数限定符形与隐式 this 形（getClass 片已修）**零回退**。
- 冻结真 javac 8 编译的正例（分配限定符形）+ 健全性负例，新增 CI 测试；再生 corpus fingerprint。

## Impact

- **代码**：`crates/jarde-java/src/init.rs` 的实例读者计算（`outside_readers` 约 1298、`verify_new` 的 `produced_by`/`readers`/`written` 约 485/681-711），以及分配限定符臂（`verify_member` 约 929-976）——后者是"检查落在实参窗口内"的另一处，须与读者门协调（root 实测 `main` 的第一道门是读者门 687，但实现者停手报告归因到实参窗口门 `jre_new_member_order`；**两者可能在不同层级各自成立，本片须自行以插桩确认第一道门并据此定落点**）。**可能触及** `src/member_inner.rs`（分配限定符的成员调用路径）。
- **不涉及**：`emit.rs`/`report.rs`（发射侧 `outer.new Inner(args)` 通道已由 DT-03 两片交付）；`guard.rs`；`crates/jarde-reader`。
- **账本**：DT-03（`declarations-types.md` 已记"部分修复"，本片落地后改为"已修复"）；`summary.md` 的 DT-03 状态归属从"有差距"改回"冻结差距已修复"（计数 45→46、1→0）。
- **验收锚纪律**：必须**双腿**（真 javac 8 与 javac 23 `--release 8`）都通过——单腿正是 getClass 片盲区不可见的原因。

## Non-Goals

- **不**放宽实例读者门对**其它**读者的判定（只认被丢弃的 null-check 三元组；用户显式 `o.getClass();` 语句仍不得被误折叠——getClass 片的负例 C 纪律沿用）。
- **不**引入 `java_release`/`major_version` 判据（两种产物 major version 均 52，且违反 `classfile.rs:139-141`"never over the compiler that produced it"）。
- **不**触碰 TWR（[twr-javac8-codegen-patrol](../../evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/README.md) 是独立的资源关闭惯用法缺口）、写访问器（EM-15）、`make` 硬编码卫生债——各自独立。
- **不**做全语料真 javac 8 重编（独立大颗粒项）。

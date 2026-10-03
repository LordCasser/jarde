# 单静态接口子折叠缺口巡查（2026-10-03）

[lambda 验收](../lambda-inline-patrol/README.md) 中发现的独立缺口（主线 `a7762a92`）。固定转录 [fixture](fixture/)（Y1 家族：单直接静态**接口**子 `Y1$StrFn`〔InnerClasses access_flags 1544=0x0608 static+interface+abstract〕；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），诊断快照 [results/member_family.txt](results/member_family.txt)。

## 表现与判别

| 输入 | 主线 Jarde |
| --- | --- |
| 单直接静态**类**子（M2$Solo，fold 片验收） | 折叠（窄通道回退进入折叠） |
| 多直接静态子（含接口 0x0608，fold 片 FV 变体） | 折叠（`StaticMembers` 通道，接口行已准入） |
| **单直接静态接口子（Y1$StrFn）** | 不折叠——`member_family.projection.state="refused"`，reason="capture proof is incomplete"；`capture.reason="static no-capture target was not proved from the class-level relation"` |

## 根因（2026-10-03 更正）

> **本节原有归因已由 `sif/` 取证否定。** 本表首行与末行的差异不是"接口 vs 类"行准入，而是
> **根类文本是否携带 lambda 伴生投影**。受控矩阵与巡逻基线重放见
> [sif/README.md](sif/README.md)，复现脚本 `sif/reproduce.sh`。

- **行准入早已存在**：`scan_family_root` 的行判据 `source_spellable_member_row` 自 `75a0114c`
  起就承认 0x0608/0x0609 接口行，早于本巡查基线 `a7762a92`。单静态接口子**已经**选出并作为
  `FamilyRootScan::Candidate` 参与证明——不是"落回没有准入的窄通道"。
- **判别量是 lambda**：`Z3`（单静态接口子、无 lambda）在本基线**已折叠**；`Y1X`（Y1 形态把
  lambda 写成等价非 lambda 形式）也折叠。`Y1M` 只把子行换成 `class`、lambda 内容与 Y1 逐字
  相同，同样不折叠。故 Y1 的增长点是"接口子"与"lambda 根"两个变量的混淆。
- **Y1 的真实链**：折叠的**根重投影门**（`project_class_source_member_fold` 要求折叠 context
  重投影等于 `root.text`，而该 context 把 lambda/数组/枚举/初始化器的投影输入全置 `None`，重投影
  得到物理文本）+ **token 锚定门**（`Y1$StrFn local1` 的覆盖段只有 `bcis={5}` = `astore_1`）。
  两门对 `class` 子同样生效（`Y1M` 亦卡在根门）。
- **一处确为接口专属的缺口**（附带发现，见 `sif/README.md` §5）：锚定匹配器漏
  `CpEntryKind::InterfaceMethodRef` owner（`WCallI` vs `WCallC`）；最小补丁多折叠 4 个 corpus 类，
  但**不能**使 Y1 折叠。

## 处置方向（更正）

`recover-single-static-interface-fold` 的**两案均不成立**（取证见 `sif/README.md` §3）：单静态行
改道 `StaticMembers` 会改变真实类 `VN3` 的报告面与退出码（增加 `anonymous_child_shape_unproved`、
exit 4），且不能折叠 Y1；窄通道"加 0x0608 准入"无码可加（准入已在），真正的接口专属判据是
`prove_static_family_target` 的"单一 `()V` 构造器"要求，补它也只是新增接口 no-capture 证书、
仍不能折叠 Y1。

真正需要的是跨出窄切片、触及共享契约的改动：折叠必须能重投影根类**已投影**的文本（把 lambda/
数组/枚举/初始化器投影输入保留进 `ClassSourceReport`，或在折叠内重建同 context），并为"覆盖段
不含命名该类型的 CP 条目"的类型 token 提供**健全**锚。需架构决策后另立切片；接口锚补丁
（`InterfaceMethodRef`）可作其中独立的一小步。

原 class 为行为基准。

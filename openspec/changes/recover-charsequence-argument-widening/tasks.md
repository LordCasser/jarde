## 1. 取证与基线（root 已完成大半）

- [x] 1.0 拒绝点、三通道不覆盖、javadoc 实现者族、jadx 有解——实测归档。（root 已完成）
- [x] 1.1 实现者全集核对：以 **release 8 javadoc** 逐一核对 CharSequence 实现者（String/StringBuffer/StringBuilder/CharBuffer；Segment 是 9+ 勿入），转录依据存证据。
      → 实测（`javap` on Corretto 1.8.0_432 rt.jar，转录存 `openspec/evidence/java-syntax-2026-10-05/widening-row-sources/`）：四行全部header 自声明；
      **两处与 spec 措辞的偏差已如实记录**：(a) release 8 的 CharSequence 实现者列表**也含 `javax.swing.text.Segment`**（spec 记它为 “9+”，实为 8 即存在）——本片按 spec 钉的封闭四行落表，`Segment` 仍拒（保守省略，负例实测）；(b) `java.lang.StringBuilder`/`StringBuffer` 的 `Serializable` 事实成立但不入 Serializable 九行（同保守）。
- [x] 1.2 重验基线：主线二进制渲染 `SB`（`join` 拒、其余恢复）；负例探针（String→Runnable、Integer→CharSequence）现状拒绝记录。
      → 实测：`SB.join` BCI 3 拒（3 引注，全类 3 处 `not recovered`）；两条负例在源级**不可产生**（javac 拒绝此类源），故以单元级表拒绝钉住（build.rs 单测）。

## 2. 实现

- [x] 2.1 按决策 1 落表（四行封闭；落点按 Open Question 1，实现者定并说明）；命中走 `cast_argument`（决策 2，零第三种呈现）。
      → 落点：`platform_interface_argument_widens`（与 `java_lang_throwable_widens` 的独立小函数形同构，四张 `const` 行表同一函数；序列化员同函数）。
      **必需伴随（spec Scenario 1 的实测推论）**：`String.join` 的**第 2 参**（`String[]`→`CharSequence[]`）需要同一事实在**数组位置**的投影——只落类行时该参仍在 BCI 3 被拒、整方法仍拒；故新增 `platform_array_argument_widens`（只把两个非数组引用组件交给这三张新表；既有 `array_reference_widens`/java.util 表/Throwable 通道的数组答案逐字未动）。
- [x] 2.2 不动 `DIRECT_EDGES` 集合表与 Throwable 通道。
      → 实测：两函数与其行逐字未改（`git diff` 只增新函数与两处调用点）。

## 3. 验证与验收

- [x] 3.1 主锚：`SB.join` 恢复 0 引注、整类 `javac --release 8` exit 0、行为一致。
      → 实测：`java.lang.String.join((java.lang.CharSequence) "-", (java.lang.CharSequence[]) arg0);`；整类 refusals=0；ignored replay（双腿）答案 `p-q/a,b,c/b/s` 与 fixture 自身 class 一致。
- [x] 3.2 零回退：集合/Throwable 扩宽既有测试全绿；负例两条仍拒；corpus 双腿扫描 diff 为空。
      → 既有集合/Throwable 测试全绿（其中集合表 `CWN` 的 `EnumSet→Set` 一行由**枚举片**的级联伴行移动，见该片 verification 与本节“移动的 pin”）；负例（`StringBuilder→Serializable`、`Segment→CharSequence`）实测仍拒；`SB`/`ST`/`GE` 之外的成员逐字未动。
- [x] 3.3 门禁全量（基线以合并态为准；flake 家族单测复跑两轮判定）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + 再生 fingerprint。
      → 见本片 verification“门禁”节；fingerprint 与 fixture 人口计数按约定再生。
- [ ] 3.4 root 独立复核：实现者全集与 javadoc 一致、表封闭不外推、零回退实测；关闭 summary.md 登记行。（留 root）

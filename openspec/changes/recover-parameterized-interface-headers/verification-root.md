# Root 独立验收（2026-10-06，合并 `bf4675ae`）

## 判据逐项

1. **diff 审查**：两处钉死的 `$` 拒绝恰在 `class_source.rs:6444` 与 `facade.rs:13761` 删除、其余判据逐字保留；`project_generic_signature` 新增第四条 `interface_only` 路径（逐接口三态：Proved 投影 / Unresolved 裸+桥可见 / Contradicted 拒头）；拼写为**参数化单递归**（`ClassNameSpelling::{SelectedSourcePath,BinaryPoolName}`——成员/注解位走原 `simple_generic_class_name` 逐字不变，类头位走新 `binary_pool_class_name`=裸头本就用的 binary 名纪律，不重拼不推断嵌套）；`facade.rs::prove_header_interface_definition` 并列接口证明器（不复用拒 `ACC_INTERFACE` 的父类证明器）；桥接缝 `header_interface_arguments` 单一事实源，桥前置改读它（`owner_generic && !carried` 才保桥可见）。
2. **两处前提更正的 root 裁定**：
   - 拼写侧：`simple_generic_class_name` 对含 `$` 名一律 `Err` 是成员域既有边界（evidence 2026-09-24 boundaries 已档），原 proposal"拼写机制无需扩展"不成立；`binary_pool_class_name` 叶子规则是**最小必要**（与裸头 `class_name` 同一纪律），单元测试钉住成员位不放宽。**追认**。
   - 平台事实：选定环境不带 JRE image（facade 注释自证），`Comparable` 定义不可解析则锚不可达；按桥准入同规（`java8_class_path_runtime` 提为共享 helper、同一 `Missing` 判据、rt.jar `b27515a6…` 转录在案）加**一条**封闭事实，门控为类名恰 `java/lang/Comparable` ∧ 实参恰 1 ∧ Java 8 class-path ∧ 干净 Missing。**追认**；扩表须按 widening 转录规另立（残余边界 2 已登记）。
3. **root 实测（合并态自建 CLI，探针全部 jar 输入 + 自述头断言）**：`Spec`/`BR$StrBox` 池形参数化头（`extends Outer$Box<java.lang.String>`/`extends BR$Box<java.lang.String>`）+ `set(Object)` 桥 hidden ✓；`IfaceImpl implements java.lang.Comparable<IfaceImpl>` ✓；`MultiIface` 部分投影（`Comparable<MultiIface>, Runnable`）✓；`Unresolved` 裸保留 ✓。
4. **定向测试（root 本机）**：`parameterized_interface_headers` 8/8、`class_source` 102/102（含改名改断言的 `the_parameterized_superclass_header_hides_the_parameter_bridge`——三向运行打 `SPEC.set(String) ran`，断言更新非删除）、`ordinary_generic_projection` 14/14、`nested_generic_header_projection` 6/6。
5. **门禁（root 合并态）**：全量 `--no-fail-fast` **3094 passed / 0 failed / 58 ignored**（与实现片一致）；fmt exit 0；CI 逐字 clippy `Finished` exit 0；`openspec validate --all --strict` **302/302**。语料 pin 迁移（reader 计数 `(735,3061,282,1827,8)`）与 fingerprint 再生随合并；corpus 双腿扫描（实现片自检先行）：loose 0 移动、family 腿 12 移动=普查 4 类+新 fixture 8 格、越界 0。
6. **CI**：合并推送后 run 为准（监控在案）。

## 残余边界（实现片登记，root 确认归属）

- 擦除契约自调用形（`ErasedCall`）：桥隐藏后 body 不可编但响亮——调用点实参重定域（invocation-argument-typing）债，非本片；
- 平台事实仅 `Comparable` 一条；其余平台接口无定义即裸头（扩表另立）；
- `single_class` 策略不解析 ⇒ 裸头（与桥准入同姿）。

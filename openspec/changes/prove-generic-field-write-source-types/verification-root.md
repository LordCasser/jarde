# 泛型字段写来源证明 — root 验收

基线 main `2dea3217`，候选为 `codex/generic-field-write-types`。root 独立构建最终 CLI，使用实际 Corretto 8u432 和 OpenJDK 23.0.1 编译原源码及完整呈现类，所有运行启用 `-Xverify:all`。实现由 Luna 执行，发布顺序、SSA 来源、既有 raw-bound 正例及最终验收由 root 复核。

## 架构与裁决

补的是现有 same-class 使用清单的写入事实。方法 Signature 在既有 class commit 阶段先结清，只有实际成功发布的参数类型才进入 writer 参数事实，其余使用物理 descriptor。字段按完整清单原子决定投影；不新增 pass、fixpoint、IR 或 Signature parser。

写来源只接纳同轮直接参数槽、直接 null 和完成正文中的直接原始分配。参数槽包含 static/instance 和 long/double 占两槽的布局，物理方法身份与字段 owner/name/descriptor/BCI 保留；改写参数、局部变量、调用、phi、未知来源以及重复 SSA BCI 不猜测类型。T/U、类/方法同名 binder 不按擦除或名字合并。预算和取消传播为停止，不能继续发布字段。

原始集合参数/分配以及已发布方法变量的显式 raw class bound，复用现有 release 8 平台关系。后者仅为保留实测已有的 `RawBoundWriter` 能力：无参数化 bound、无 interface bound、bound 擦除与物理参数相符，目标为参数化 Class。没有引入通用泛型子类型或上界推理。

见 [提交位置审计](results/publication-placement-audit.md)。参考了本地 JADX 的 SignatureProcessor、TestGenericFields 和 TestConstructorGenerics；它的字段 Signature 解析不等于最终 RHS 源码赋值证明。Object-erased source cast 本来就可能不存在于 bytecode，不能凭 Signature 为输出猜 cast。

## 完整对照

独立回放脚本：`python3 openspec/changes/prove-generic-field-write-source-types/results/replay-root.py --candidate /tmp/jarde-generic-final-cli --baseline /tmp/jarde-generic-baseline-cli`。CLI 均由 root 构建；真实输入每次由原源码重编，外部 driver 不进入 decompiler 输入。候选编译/运行 classpath 只包含候选重编类和 driver，禁止原 jar 掩盖缺失类。

| 实际编译器 | 对照类数 | 基线完整编译成功 | 候选完整编译成功 | 候选验证与行为一致 |
| --- | ---: | ---: | ---: | ---: |
| Corretto 8 | 23 | 10 | 22 | 22 |
| OpenJDK 23 (`--release 8`) | 23 | 10 | 22 | 22 |

每次 class-source 请求校验自述头和非空完整类，CLI status 0/4 不作为编译成功判据。类身份、stage、物理 field/method item 在基线与候选之间相等。机器结果见 `results/root/summary.json`，逐类 Java/JSON、javac、运行及反射 diff 在该目录。

完整泛型反射（所有声明参数及字段）必须一致的六族：TypedSetter、NullSetter、RawListField、真实 Deferred 同类 writer SCGA、原有 Map 初始化 SCGBCompat、已发布 raw-bound RawBoundWriter。两腿全部一致。ListWrong、RawParamArray、StaticRawField 另有字段反射一致，但其探针只读取字段，不能计为全方法反射验证。

Hold/ObjectHold/ObjectSetter/CrossSetter/MixedSetter/ShadowSetter、数组写入及其他未知来源恢复为源码可赋值的擦除字段；完整类编译及行为一致，允许字段 Signature 减少。Hold 的 constructor 参数仍为 Object，不能声称构造器泛型反射已恢复。ArraySetter 的未发布 T[] 参数仍为 Object[]；DeferredSetter 的 private sink 参数反射仍由 T 变为 Object，是基线已有能力边界。

SCGB 新取证版本的 main 有原有 refused-body，候选与基线均不能整类编译；它不计为成功恢复。原有测试中的 Map initializer 使用不同可恢复 main，独立以 SCGBCompat 重放并确认完整编译、行为及反射都保持。

JADX 对照冻结了前 14 族、两个真实编译器输入，完整原源码/class/javap/JADX/Jarde 基线及诊断在 `openspec/evidence/generic-holder-write-boundaries`。JADX 1.5.6 完整重编成功数为 9/14（JDK 8）和 8/14（JDK 23），失败实因与未覆盖边界见该 README。这是选定字段写边界的结果，不是整个泛型单元覆盖率。

## 门禁与接续

本地验收：fmt 与 CI 同口径 all-targets/all-features clippy 通过，class-source 单元 32 项、附近字段集成 15 项通过；两固定 seed `5350648285461741569` / `5350648285461741570` 各 3,235 passed、0 failed、93 ignored。显式 ignored P3 3 项、functional-constructor 整类 1 项、既有 bound-receiver 整类 1 项全部通过；strict OpenSpec 320/320。原始日志在 `results/local-gates`。JDK 25 instruction-boundary oracle 须由 CI 的 JDK 25 环境验收，本地 8/23 不冒充 25。最终提交后远端结论须核对最新 main HEAD 的 CI；不把上述本地结果当成远端已绿。新增 CI 测试直接嵌入 javac8/javac23 五族冻结 jar（10 输入），验证完整类重编和独立运行；既有 read-consumer gate 不放宽。预算用例验证 25 步精确成功、24 步停止及提前取消。

class-scope 构造器 Signature 恢复单独排队：现有 constructor candidate 只覆盖空体或原参数转发，不涵盖初始化后字段赋值。需要明确同一 class binder、完整候选参数使用、初始化 this 接收者及字段身份，不能把未发布 constructor Signature 偷放进已发布 writer 集合。该能力本片未实现。

额外边界巡查发现一个明确的保守退化：`RawOtherWriter<T>` 的 static `put(RawOtherWriter raw,Object value) { raw.v=value; }` 在基线保留字段 T，候选将字段保留为 Object；两版都能完整编译。原始 receiver 下 `raw.v` 的 member-selection type 是 Object，不能只按字段声明的 class T 计算实际赋值目标。本片未证明 receiver 实例化，不将这个不在冻结族内的质量缺口混入实现；该输入保存在 `results/follow-up/raw-receiver`，接续必须单独立项并验收字段反射，不能宣称本片保住了所有历史泛型投影。这不是新编译错误，但确实减少了该类的字段 Signature。

# Root 对 String switch 的独立验收（2026-09-24）

Root 审读了 `stringswitch::prove` 的同方法哈希、literal、SSA 用途、入口和异常处理器约束，以及 `region::project_string_switches` 的成对区域原子替换与 `StringSwitch` 块归属；`SwitchLabels` 只在已证的 enum 或 String 标签之间选择，普通整数 key 保留为证据。结构证明失败不改两层原区域。证书已成功、但 builder 后续拒绝 selector 的分支会引用合并后的整个区域；它没有伪造 Java 行为，却可能比旧两层正文更少恢复。未找到 JVM 可验证且旧两层可执行、新路径才退化的反例，此质量边界保留在后续巡查，不为假设性风险增加回滚机制。

[2.2–3.1 整类重放](verification-2.2-to-3.1.md)由 root 用最终 CLI SHA `d3bd76e99733d3d7be66559564e0f0f22452a9b156e1e1ce6744d519c98ee1eb` 独立执行：四份正例的原/JADX/Jarde 共 51 行一致，Jarde 单层；`StringSwitchMiddleDefault` 比 JADX 的两层恢复更完整。三种非法折叠候选保留两层、原/Jarde 逐行相同。负例 `ExtraHashUse` 的额外 hash 消费按独立 6 行 runner 再检，奇数 hash 输入返回 41，没有副作用丢失。[2.4](verification-2.4.md)另固定 29 个折叠 BCI 来源、正文一致、限额和取消。

Root 在独立 Cargo target 重跑：`p3_string_switch_projection` 6/6、`jarde-java --lib` 139/139、`p3_char_switch` 1/1、`p3_switch_fallthrough` 2/2、`class_source` 的 enum 投影 2/2。`cargo fmt --all --check`、`git diff --check`、`openspec validate recover-string-switch --strict` 均通过。target 已用 `cargo clean --target-dir` 移除 5144 个文件、1.8 GiB。

两个全局门禁仍红，属于多个在途变更共享的范围，未混入字符串投影修复：

- `p5_corpus_fingerprint::corpus_files_match_the_recorded_fingerprint` 报 60 个未列入清单的其它夹具。Root 将本轮新增的 `ExtraBucketEffectRunner.java` 放入 `openspec/evidence/`，从 `tests/fixtures` 移出；余下 60 项跨 catch/conditional/switch 等已有工作，不宜在本变更中盲目重录 manifest。
- `cargo clippy -p jarde-java --lib --locked -- -D warnings` 报 7 项既存 lint：`enumswitch.rs` 两处 useless conversion、`region.rs` 一处 too many arguments 和一处 type complexity、`report.rs` 三处 needless option as deref。均不在本轮 String switch 新增代码，需独立清理。

以上明确了 3.2 的实际验收边界：功能与受影响回归、格式和规格通过，整仓指纹与严格 Clippy 尚不能声明为绿色。

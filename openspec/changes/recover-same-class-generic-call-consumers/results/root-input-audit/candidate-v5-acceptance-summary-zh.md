# CLI5 固定候选重放摘要

候选 CLI 为 `/private/tmp/jarde-generic-calls-candidate-v5-cli`，SHA-256 `010e9f1ecd82a1f2c7c4dcd49010afc787d3980a0194e9fc6a1d12994132dac5`；构建输入身份指向 frozen v36 snapshot，归档 SHA-256 `870589b655d855c7eedf6951c230bd8a69158f98b4ea54f54c14da008d9dc9a2`。固定 acceptance runner SHA-256 `a19895e70ef306a3024132a3da7c07cf891ea87e5ef0e25e01413989514ce836`。

本轮固定 140 个矩阵输入加 4 个 Nested 输入，共 **144 行**；矩阵覆盖 35 个 family，物理 class-source 输出 148 份。两组 runner exit 均为 0，manifest 完整性错误为 0。独立矩阵/Nested verifier 分别核对 140/4 输入、逐文件哈希和实际转录，均通过。

候选计数：编译 144/144，Probe 144/144，行为一致 144/144，完整反射匹配 96/144。功能恢复 100，拒绝控制谓词通过 8，边界控制谓词通过 8。总体矩阵状态是 **needs-review**；这不是 GC-09/10 的结论。

11 个重点 family 的固定判据事实：

| Family | 角色 | 编译/Probe/行为 | 反射匹配 | 固定判据结果 |
| --- | --- | ---: | ---: | --- |
| UnknownIncoming | mixed-incoming-control | 4/4 · 4/4 · 4/4 | 0/4 | CONTROL_REVIEW_ONLY；留 root 裁定 |
| CycleRelay | finite-cycle-refusal | 4/4 · 4/4 · 4/4 | 0/4 | CONTROL_REVIEW_ONLY；留 root 裁定 |
| MethodHandleUse | bootstrap-boundary | 4/4 · 4/4 · 4/4 | 0/4 | CONTROL_REVIEW_ONLY；留 root 裁定 |
| RawOwnReceiver | raw-receiver-refusal-control | 4/4 · 4/4 · 4/4 | 0/4 | CONTROL_REVIEW_ONLY；留 root 裁定 |
| ReboundOwnReceiver | rebound-receiver-refusal-control | 4/4 · 4/4 · 4/4 | 0/4 | CONTROL_REVIEW_ONLY；留 root 裁定 |
| InheritedUnknown | inherited-owner-boundary | 4/4 · 4/4 · 4/4 | 0/4 | CONTROL_REVIEW_ONLY；留 root 裁定 |
| VarargsCall | varargs-boundary-control | 4/4 · 4/4 · 4/4 | 0/4 | CONTROL_REVIEW_ONLY；留 root 裁定 |
| SameErasureBinder | distinct-binder-boundary | 4/4 · 4/4 · 4/4 | 0/4 | 拒绝控制谓词 4/4 pass（不是 API 恢复成功） |
| MultiUseResult | all-consumers-control | 4/4 · 4/4 · 4/4 | 0/4 | 拒绝控制谓词 4/4 pass（不是 API 恢复成功） |
| BridgeUnknown | bridge-control | 4/4 · 4/4 · 4/4 | 4/4 | 正控制 pass；feature 0/4，boundary 4/4 |
| IncompleteSite | positive-conditional-single-call | 4/4 · 4/4 · 4/4 | 4/4 | 正控制 pass；feature 4/4，boundary 0/4 |

本矩阵 GC-03、GC-05、GC-08 family 判据需 root review。七项拒绝控制四腿的 compile/Probe/行为均通过，但不据此判定拒绝/API 边界正确；SameErasureBinder 与 MultiUseResult 只记录固定拒绝控制谓词结果。BridgeUnknown 与 IncompleteSite 作为正控制保留，不混入拒绝样本。

44 条逐腿控制事实及哈希见 [candidate-v5-control-facts.json](candidate-v5-control-facts.json)，SHA-256 `fbc51c9c9f7e8b54f8dff4978330bc0399bb004c4c4dd55e2c533068e9dcc0d4`。完整 streams、commands、runner snapshots 与逐文件 hash 见 [candidate-v5/run-manifest.json](../candidate/candidate-v5/run-manifest.json)；矩阵和嵌套细节见各自 manifests。

## 1. 诊断与基线

- [x] 1.1 定位 TWR saved-return 声明的实际决策点与 Object 来源（未走 written_type / SSA 宽型 / 硬编码），记录证据；冻结 T2 基线输出与既有 null 期望的对照。
- [x] 1.2 构造并冻结至少四个 verifier 有效变体：String 字面量、int 字面量、构造点、调用返回的保存值（外加 null 对照），记录实现前后输出。

## 2. 细化实现

- [x] 2.1 saved-return 声明经 `written_type` 细化、失败回退（design 决策 1）；T2.voidBodyReturnInside 输出 `String local1 = "in";` 且整类 `javac --release 8` 通过；null 期望不变；四变体类型正确。
- [x] 2.2 回归：`p3_java_recovery.rs` 全绿、TWR 家族与 finally saved-return 证书（Test5/7/9、Tf、17a/17b）零回退；预算/取消原子性不受影响。

## 3. 验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 同款 Clippy 新码零新增、`openspec validate --all --strict`、diff check；磁盘低于 15Gi 先 `cargo clean`，完成即清。
- [x] 3.2 T2 三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 正常与注入异常路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核决策点落位、细化与回退边界，更新 CF-17 巡查账本（勾销已登记缺陷）。（root 于合并主线 0e1ef5ca 复核：诊断坐实 frame 层 ldc 常量 Unknown 宽型 + Store 步进缺口，`constant_of_value` 落在 `written_type` 复用通道；T2 输出 `java.lang.String local1 = "in";` 整类可编；null/Class 变体边界正确（Class 字面量→java.lang.Class）；全仓 2726/0、fmt/openspec 226/226。17a 三处旧钉死更新与机械补丁删除复核认可——旧钉钉的是本缺陷自身，与 spec 直接冲突，补丁本为披露性临时手段。）

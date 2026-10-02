# 限定 super 默认方法调用巡查（2026-10-02）

接口 default 冲突调解域巡查（主线 `d8dbb087`）。固定转录 [fixture](fixture/)（F1 菱形/重抽象家族 fam.jar + F2 静态块抛错对照；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 o1.out（`AB`/`I:A`）。

## 结果矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| default 接口本体（A/B）、重抽象子接口（C `@Override String name();`）、调用方 F1、F2 静态块条件抛错 + `<clinit>` 包装异常 | 全部恢复 |
| **`A.super.name()`——限定 super 默认调用（invokespecial InterfaceMethod）** | 拒绝：单限定（`F1$Reabstract$Impl`）与菱形双限定（`F1$Diamond`，`A.super.name() + B.super.name()`）同败 |

拒绝文案（build.rs:21191）：`the interface-special target F1$A.name … has no selected proof of a legal source qualifier and unique default binding` —— **通道已存在**（interface-special 判定与专属拒绝），是其"合法限定符 + 唯一默认绑定"证明选择失败。

## 根因假设（取证义务）

判定要求证明 (a) 限定符接口是调用类的合法直接超接口、 该方法在该接口为 default（非 abstract/static）。两事实均需**跨类读取**（调用类 header 的 interfaces 数组 + 限定符接口的方法成员标志）——单类输入下 `F1$Diamond.class` 自身 header 含 `implements A,B`（同 jar 快照内可读，snapshot-hierarchy-widening 切片先例），接口方法标志同 jar 内 `F1$A.class` 可读。第一个取证义务：读 build.rs:21191 附近的证明选择逻辑，确认失败环节是"证明根本未尝试读取快照内接口事实"还是"判据过严（如要求菱形冲突才启用）"。

## 处置方向

`recover-qualified-super-default-calls`：interface-special 证明接入快照内跨类事实——限定符为调用类直接超接口（header interfaces）∧ 目标方法在该接口成员表中为非 abstract 实例方法（default）时，呈现 `X.super.m(args)`；参数与结果按既有 invocation 通道。菱形双限定（A+B）与单限定同判据；接口方法不可读（快照外）保持现拒绝并登记。判据为快照 header/成员事实（结构），无新机制。

原 class 为行为基准。

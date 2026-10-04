## ADDED Requirements

### Requirement: 真 javac 8 的 TWR 关闭序列 SHALL 恢复为源级 try-with-resources

当方法的异常表与 CFG 呈现真 javac 8 的 try-with-resources 关闭序列几何——**primary-exception 空槽引导**（`aconst_null` + 局部槽）、**守卫式主 close**（资源 `ifnull` 与 primary 槽 `ifnull` 双守卫）、**重抛-再捕获回路**（primary 异常存槽后 `athrow`，被 `any` 行再捕获进入 close 抑制链）、以及**抑制链 handler 块**（`addSuppressed` 路径，带自己的 `Throwable` 行与 `any` 行）——且资源初始化/使用/返回的既有 `Shape::Resources` 事实成立时，系统 SHALL 把该语句恢复为源级 TWR（`try (R r = …) { … }`），呈现与 javac 9+ `--release 8` 产物同构（不含 `addSuppressed` 等编译器合成文本）。

识别 SHALL 以**结构事实**（块集/异常表行集的 CFG 关系）表达；SHALL NOT 引入 `java_release`/`major_version` 或任何按产出编译器分支的判据（两形产物 major version 均 52）。既有 javac 9+ 形的 `Shape::Resources` 判据 SHALL **逐字不变**——本能力只能以**新增**几何事实（在 javac 9+ 形上恒假的抑制链/守卫事实）容纳 javac 8 形，任何对既有判据的放宽均不允许。抑制链块与守卫 close 是编译器合成、源级不可见的证据，SHALL 不在呈现中出现。

削弱任一几何事实（缺一条 `any` 行、守卫目标错乱、抑制链断头）SHALL 保持既有拒绝——新判据是必需的而非装饰。

#### Scenario: 真 javac 8 单资源 TWR 恢复

- **WHEN** 真 javac 8（Corretto 1.8.0_432）编译的 `TR.one()`（单资源，`try (TR r = new TR(n)) { return r.use(); }`，48 指令/5 异常表行含 2 `any`）经 `class-source` 呈现
- **THEN** 恢复为 `try (TR local1 = new TR(arg0)) { … }` 形，方法 0 引注（修复前整方法 `not recovered`，整类 6 引注）；整类渲染源集 `javac --release 8` exit 0，`java -Xverify:all` 运行输出与原 class 逐行一致

#### Scenario: javac 9+ 形零回退

- **WHEN** 同一 `TR.java` 以 javac 23 `--release 8` 编译（22 指令/2 行、无条件 close）与既有 `Shape::Resources`/finally 家族全部测试经呈现
- **THEN** 渲染文本与修改前逐字节相同；全部既有测试通过——既有判据未被稀释

#### Scenario: 几何削弱仍拒绝

- **WHEN** 合成探针削弱任一几何事实（删除一条 `any` 行 / `ifnull` 目标互换 / 抑制链断头）
- **THEN** 保持既有拒绝——每条新几何事实都是准入必需

#### Scenario: 呈现不含编译器合成文本

- **WHEN** 恢复后的 TWR 呈现被检查
- **THEN** 不含 `addSuppressed`、抑制槽、守卫 close 等编译器合成结构的源级文本——与 javac 9+ 腿呈现同构

#### Scenario: 判据不得按编译器版本分支

- **WHEN** 审查本片生产 diff
- **THEN** 无 `java_release`/`major_version` 或等价版本分支；识别只取决于异常表/CFG 的结构事实

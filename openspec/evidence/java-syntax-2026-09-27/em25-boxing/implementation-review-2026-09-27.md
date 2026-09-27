# EM-25 窄装箱实现复核

实现仅在 `Builder::return_expr` 识别同一 SSA 中精确的静态 `Boolean.valueOf(Z)`、`Integer.valueOf(I)`、`Character.valueOf(C)`。唯一输出必须直接由同块 `return` 消费；唯一实参必须是正确类型的整数字面量；返回类型必须是对应包装类或 `java.lang.Object`。所有范围、descriptor、owner、消费者或返回目标不匹配时继续使用现有调用发射/回退。

获证字面量复用现有表达式呈现：布尔 `0/1` 转为 `false/true`，char 由既有 `narrowed_literal` 呈现。literal 保持直接 BCI，省略的 `valueOf` BCI 作为派生来源，return BCI 继续由 return 语句持有。调用和实参唯一消费者检查经共享预算与取消检查器完成；停止通过现有 `ValueRenderFailure::Stop` 向上传播，表达式不会在证明中途提交。

固定回放扩充了整数 `-128/127`、越界 `-129/128`，char ASCII `127` 与越界 `128`，以及 `true/false`。`Byte`、`Short`、`Long` 保持显式调用；`Number` 返回目标和嵌套调用消费者也保持显式调用。独立 `SecondConsumerCases` 验证同一装箱结果还被第二个物理消费者读取时，输出保留调用和 `@bytecode` 回退。

两轮回放的这些字段完全相同：固定 JADX 源码 HEAD、六个回归测试及四个生产文件 SHA、Jarde CLI SHA、所有输入/字节码 SHA、三方源码 SHA 和运行 stdout。原/JADX/Jarde 的完整 `BoxingAudit` 与共同 Runner 均通过 `javac --release 8`、`java -Xverify:all`，输出一致：

- `BoxingAudit.java`: `1b897d382f267b4c0ffcb75499699022c14bbc25e5668fefcdb1de1d38b57976`
- `Runner.java`: `338a87d555c51b1abb2a143b916458f35c2886564a61fb4c46a866d3a010c092`
- `SecondConsumerCases.java`: `81957a02ddc30849bf76fb3437cf3e22241d01a11b0f2564254def800f148b7a`
- Jarde CLI: `79600bb2b7184db026d17ce6aa64a8bc93e128af777850eaf852a2691585e9e0`
- 原/JADX/Jarde 重建源码 SHA：`1b897d382f267b4c0ffcb75499699022c14bbc25e5668fefcdb1de1d38b57976` / `8048a492ecf94f80303f0681934372e3f4a58e7245d08233410fc9d7e866ff4f` / `a948bea60b39185552124c420d93db03b9f4c50843aee0cd68af7d1df2c85bfe`

重放文件、源码映射及 source SHA 详见 [`replay/narrow-safe-boxing`](replay/narrow-safe-boxing/)。固定 JADX 源 checkout 为 `2fb1b16386941660fda07e9017285aec40fcb37f`，本机执行的 JADX 命令版本为 `1.5.6`；回放脚本逐项核验了固定 checkout 中的源码哈希。

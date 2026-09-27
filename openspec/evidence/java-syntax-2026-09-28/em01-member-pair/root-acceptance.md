# EM-01 `Shape` 两成员联合恢复：root 验收

固定输入来自 JADX `TestClassGen` 所需的 `Shape` 子形态，JADX checkout 固定在 `2fb1b16386941660fda07e9017285aec40fcb37f`。本次只验 `Shape` 根类的直接成员 `I` 与 `A`；`Generic.A` 的泛型 Signature、父接口与 bridge 不在联合声明证书内。

我独立审阅了 `scan_family_root` 的两条直接 `InnerClasses` row、两份 child self relation、物理 class/method flags、准备阶段的两份完整报告，以及同一次根源码投影。固定 class SHA 和重放命令见 [`tests/fixtures/em01-member-pair/README.md`](../../../../tests/fixtures/em01-member-pair/README.md)。在新的专用 Cargo target 构建 CLI 后，我运行 `tests/fixtures/em01-member-pair/replay.sh`，将原 class、固定 JADX 源码、Jarde 根源码分别用 `javac --release 8 -g:none` 重编，再用同一 Runner 执行 `java -Xverify:all`；三份结果逐字相同，均为 `2:1`。根源码各有一次 `I`、`A`，三个抽象方法为无正文分号声明，`A()` 使用源级构造名；两个物理 child 报告仍可独立查询。

八个可解析且 JVM verifier 有效的近邻，包括缺 child、冲突 self row、第三 child、接口 default/static Code 方法、字段、Signature 和根构造器使用，均未发布半组声明。`Generic.A` 继续拒绝。`member_family_identity` 20/20 通过，覆盖两 child 顺序、六段派生来源及物理锚点、输出预算停止与旧单成员路径。OpenSpec strict 和 diff check 通过；实现代理另运行 class-source 87、generic family 3、local-name 3 项及 workspace check、fmt。独立回放结果保存在 `/private/tmp/jarde-em01-root-replay`，专用编译目录验收后清理。

旧 EM-01 `multi` 回放的断言已随现状调整并独立重跑通过：`Shape.I` 和 `Shape.A` 应出现，`Generic.A` 仍缺，因而完整 `multi` 项仍不能编译。此验收只确认准确的两声明组合，不将 EM-01 整单元标为追平。

## 1. 前置证书与捕获来源

- [x] 1.1 先复核 DT-25 已验收的同类 helper 身份、body、整类引用 census 与原子提交接缝，确认 DT-26 复用这些入口；用 DT-25 固定回放及现有拒绝测试证明无捕获行为未退化
- [x] 1.2 从固定静态/实例站点证明 bootstrap handle、owner、descriptor、flags 与物理 helper 精确一致，并把一个 `int` 捕获或 `this`+`int` 捕获按 receiver/参数顺序绑定；用错 handle/flags 的 classfile 反例验证拒绝，并用非交换减法验证捕获/SAM 顺序
- [x] 1.3 以完整 enclosing 方法的 SSA/slot 写入事实证明捕获只来自直接 `this` 和 effectively-final `int` 参数；用效果性表达式与 phi 来源反例验证不移动创建时求值；对所有字节码指令扫描 Store/iinc 拒绝可观察的参数重赋值

## 2. Helper body 与类级原子投影

- [x] 2.1 将静态直线算术和实例 `this.number()` 加法 helper 的完整 AST 表达式映射到 lambda 体，保留站点/方法指令来源；以固定源码和复杂 body 拒绝测试验证调用时语义
- [x] 2.2 沿用 DT-25 全类引用 census 和统一原子提交；捕获拒绝、handle/flags 不匹配均保留物理 helper，物理方法记录仍可查
- [x] 2.3 将捕获扫描、body 替换、输出和取消接入同一预算；以低预算和已取消请求验证无半份 lambda 内联/省略

## 3. 三方重放与验收

- [x] 3.1 更新固定 `dt26-lambda-capture/replay.py` 为修后模式，原/JADX/Jarde 的完整源码与原 `Runner` 都通过 `javac --release 8` 和 `java -Xverify:all`，输出逐字为 `7:-1`，Jarde 类源码不含 synthetic helper 声明/调用
- [x] 3.2 跑相关 lambda/class-source 回归、`cargo fmt --check`、workspace check 与 `openspec validate recover-proved-captured-lambda-bodies --strict`，并在报告中记录首片之外的捕获变体与独立门禁债务

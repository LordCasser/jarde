# IO 资源 across-finally 恢复（recover-io-resource-finally）

## Why

[io-wrapping 巡查](../../evidence/java-syntax-2026-10-05/io-wrapping-patrol/README.md)（local-scope 第 13 锚）：`BufferedReader r = new BufferedReader(new InputStreamReader(new FileInputStream(path), "UTF-8")); try { while ((line = r.readLine()) != null) {…} } finally { r.close(); }`（IO 样板=真实代码最高频资源管理形）两方法整方法 crosses 拒。`recover-lock-guard-loop-finally`（锚 12）已交付 guard 层证书，但其形状是**单 any-catch 行**；IO 形是**每方法 2 行**（`[25,45)→52` + `[52,54)→52`）且资源句柄（非保存返回值）跨 body 循环读 + finally 关闭双区域。jadx 完整解。

**两部件**（巡查预记 + root 2026-10-07 javap 核实）：
1. **2 行 resource-guard 形状**：lock-guard 证书的多行扩展——行集覆盖同一 finally（close），资源局部 SSA 同一（body 读与 finally 关同一句柄），body=循环+会抛调用，保存返回值在体内；
2. **两行平台引用宽化表行**（`platform_reference_argument_widens` 的 java.io 行，javap 转录）：`java.io.FileInputStream → java.io.InputStream`（InputStreamReader ctor 实参位，javap `InputStreamReader(java.io.InputStream)`）与 `java.io.InputStreamReader → java.io.Reader`（BufferedReader ctor 实参位，javap `BufferedReader(java.io.Reader)`）。

## What Changes

- guard 证书扩展到行集形（≥2 行同 finally/同资源）：close 接收者 SSA 同一性同锚 12 纪律；行集的外层行覆盖范围与内层行不交叉（复用 `CrossingExceptionRegions` 判别）；
- 宽化表行入 `platform_reference_argument_widens`（与 java.util 行同表同门同渲染 `cast_argument`）——零新机制；
- MVP：单资源单 finally、close 无参形、FileReader 手动 char 循环形一并（同判据）；多资源嵌套 try、close 带返回值形登记边界。

## 硬不变量

1. lock-guard 单行锚（LK 三法）与 TWR 系列渲染逐字节不变；
2. 既有宽化表行/门/渲染零触碰（纯增行）；
3. 不得产出"可编译且行为不同"文本（close 时机=控制流事实）。

## 验收

- IO 两方法恢复（0 crosses 族拒绝），整类剥离编译 exit 0、`-Xverify:all` 输出与原一致（巡查值）；LK/lock-guard 套件零回退；
- 门控实验先行（行集扩展单独翻转 IO、LK 不动）；全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：资源句柄跨 finally 的 IO 读取形按源码形态呈现，方法行为完整。

# 循环测试 copy-and-store 的呈现（recover-loop-test-copy-store）

## Why

[io-resource-finally 片登记边界](../recover-io-resource-finally/)：`IO.readAll`（`while ((c = fr.read()) != -1) { … }`——FileReader 手动 char 循环，IO 样板的第二形）仍整体引注。其残余是 copy 族的**纯度判据**面：javac 把 `(c = read()) != -1` 降低为 `invoke read; dup; istore c; ldc -1; if_icmpne`——dup 值的 store（`c`）与测试消费并存于**循环测试位**，正是 `recover-dup-store-conditional` 的双读者形状在 loop-test 位置的表现；且该局部声明位于 resource-guard 证书呈现的 guard 体内（声明的区域归属面）。jadx 完整解。这是 io 域最后一个残余方法（countLines 已恢复）。

## What Changes

- dup-store 双读者判据扩展到**循环测试位**：store 目标的读者全部是测试（及已证提升位）时，测试表达式按 `(c = read()) != -1` 源码形呈现（store 位置=测试操作数位置，求值序零移动）；
- guard 体内的循环局部声明位：声明归属于 guard 证书已呈现的体（与 `resource_guard_lead` 的提升呈现协调）；
- 多读者/不可观察性违反形保持拒绝逐字；既有 dup-store/快照/guard 判据逐字不动。

## 硬不变量

1. copy 族/快照/dup-store/guard 族全部锚渲染逐字节不变；
2. 循环测试求值序=字节码序（store 即测试操作数位，无移动）——编译且行为不同的文本不可产出；
3. countLines 与 LK/IO/nested-lock 锚零回退。

## 验收

- `IO.readAll` 恢复（IO 类全恢复，0 引注），剥离编译 exit 0、双腿文件读取驱动（含 EOF）输出与原类一致；
- 门控实验先行；全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：循环测试位的赋值-比较复合（`while ((c = read()) != -1)`）按源码形态呈现，方法行为完整。

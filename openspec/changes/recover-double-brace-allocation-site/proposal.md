# 分配点双括号初始化形（recover-double-brace-allocation-site）

## Why

[double-brace 巡查](../../evidence/java-syntax-2026-10-05/double-brace-patrol/README.md)（第 12 个新证窄缺口）：`new ArrayList<String>(){{ add(s); }}`（双括号初始化——集合构建高频惯用法）的伴生类在**分配点单点使用**时可呈现 jadx 式源级双括号形。`recover-capture-ctor-super-order`（A 路径）已交付 ctor 重排使文本可编译（`new DB$2(arg0)` + 重排体）；B 路径是**源级还原**：识别"匿名子类 + 纯实例块体 + 单点使用"模式，在分配点直接写 `new ArrayList<String>(){{ add(s); }}`——更贴原源码，绕开 ctor 排序与伴生名问题（jadx 同形）。

**价值定位**：呈现质量升级（A 路径已保证正确性）；关闭巡查登记的第 12 窄缺口。

## What Changes

- 分配点呈现增加双括号形：伴生类满足（a）匿名子类（池名 `X$N`）、（b）体=纯实例初始化块（无方法声明、无字段声明）、（c）分配点单点使用（SSA 单消费者=该分配）、（d）超类可拼——在**使用点**呈现 `new Super(args…){ { body… } }` 双括号形，伴生类从文本中消隐；
- 判据复用既有匿名类内联证据（`inline-proved-anonymous-*` 家族的分配点身份/单点使用证明）+ 实例块纯度（A 路径的 val$ 纪律同源）；
- 捕获形（有 val$ 字段）在 A 路径重排已证时同样内联（body 里的捕获读呈参数/局部名）；
- MVP：超类为平台集合/显式类、无捕获或已证捕获；负例（伴生有方法/多分配点/超类不可拼）保持 A 路径文本（可编译）不变。

## 硬不变量

1. A 路径全部锚（DB 家族可编译+行为一致）渲染**语义等价**——B 形替换 A 形时行为对照逐字一致；
2. 双括号形体的求值序=ctor 序（实例块在 ctor 语义内）——顺序敏感控制件全跑；
3. 伴生消隐仅在四判据全过时（否则 A 路径文本不变）。

## 验收

- 巡查 DB 锚 + 捕获形呈现双括号源级形，整类剥离编译 exit 0、`-Xverify:all` 输出与原一致（`2/z`）；A 路径负例文本不变；
- 门控实验先行；全门禁 + oracle ignored 腿 + corpus 指纹。

## Capabilities

### Modified Capabilities

- `java8-recovery`：分配点单点使用的纯实例块匿名子类按双括号源级形呈现。

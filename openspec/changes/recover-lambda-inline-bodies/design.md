## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/lambda-inline-patrol/README.md)：Y1 四 lambda 位点 + 三伴生方法（`lambda$viaLambda$0` 等）与冲突编译记录（jx/Y1.java）。既有通道：invokedynamic/LambdaMetafactory 证明（lambda 语法呈现已建——定位其呈现位点与伴生方法的装配关系是**第一个取证义务**：伴生方法当前作为普通 private static 成员进入类文本，lambda 位点经 MethodHandle 引用它）；`type-immediate-functional-receivers` 等既有测试零回退边界。

## Goals / Non-Goals

**Goals:** 直线体内联 + 伴生隐藏（单用途）；复杂体非冲突名伴生；Y1 重编行为一致；无 lambda 类零回退。**Non-Goals:** `X::m` 方法引用语法呈现（行为已对，后续片）；复杂体（多语句/分支）内联；串行流 API 语义重排；捕获 this 的成员方法伴生（验一形，按单用途与直线判据自然覆盖则收，否则登记）。

## Decisions

1. **内联判据（结构事实）**：伴生方法体=单 return 表达式或直线语句序列，形参在体内按位只读出现（无重排/别名），lambda 位点=该伴生的唯一引用（MethodHandle 单用途）→ 内联（表达式形 `p -> body`，参数名沿用伴生形参名）；任一不满足走重命名保守分支。
2. **重命名保守**：`lambda$name$N` → `lambda$name$N$jarde`（同 class 无此名冲突——$jarde 后缀 javac 不再合成）；lambda 位点调用同步重写；呈现层规则（证明不动）。
3. **验收锚定**：Y1（`hi!`/`45`/`[b, aa]`/`8`）+ 变体（双参比较器、捕获局部、多 lambda 同方法、复杂体伴生〔分支〕走重命名）；负例（伴生多用途〔手工字节码双 lambda 引用同方法〕→ 不内联不隐藏、重命名也不做〔保持现呈现并登记〕）；无 lambda 类 diff 逐字不变。

## Risks / Trade-offs

- **参数位绑定错位（J/D 双槽）** → 绑定按伴生描述符形参序与 lambda 位点参数位一一对应，双槽按类别计；测试钉死。
- **内联改变求值序**（体内有副作用调用）→ 直线体语义等价内联（表达式即原语句序）；非直线保守。
- **伴生方法还被反射引用**（Java 8 无此形态，理论外）→ 单用途证明覆盖。

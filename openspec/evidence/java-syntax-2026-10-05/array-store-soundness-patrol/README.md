# 字段数组副作用存储健全性巡查（2026-10-05 root）——critical 第 7 锚（第三诊断族）

## 发现：ArrayList.add 形状可编译且存储静默丢弃（第一不变量违反）

`void add(Object t){ elems[size++] = t; }`——**集合代码最常见单条语句**——渲染体仅 `return;`：去注释 javac exit 0、行为 **`null/null` vs 原 `x/y`**（存储与 size 递增全部静默丢失）。javap：`aload_0; getfield elems; aload_0; dup; getfield size; dup_x1; iconst_1; iadd; putfield size; aload_1; aastore`（字段数组的 dup_x1 副作用下标舞蹈）。诊断族：**"the dependency chain from BCI 1 to final consumer 16 is not bounded"**。

## critical 族全景（7 锚 / 3 诊断族）

| 诊断族 | 锚 | 实证 |
|---|---|---|
| 旧值 store（"value local X held at BCI M"） | `i=i++`/`i=i--`/`a[i]=i++`/`i+=i+++1` | 6≠5/4≠5/2≠102/6≠11 |
| copy（"copy at BCI N has no proved local assignment"） | `flags \|= 1<<bit` void/值消费 | 位操作全丢（false/…/0） |
| **依赖链（"dependency chain … not bounded"）** | **`elems[size++] = t`** | **null/null vs x/y** |

**泛化结论加固**：三族共同性质 = 语句级效果（存储/递增）被引注吞掉后剩余文本碰巧可编译且行为不同。守卫必须覆盖全部三族（按诊断文本族识别或按"语句效果未被呈现"的结构判据）。

## 同巡查健康面（负结果）

- `(T[]) new Object[n]`：erasure 无操作 checkcast 被略——**语义等价**（擦除目标即 `[Ljava/lang/Object;`，呈现 `return new Object[n]` 行为同）；调用点 checkcast（`[Ljava/lang/String;`）如实保留（GA main 双版同 CCE）；
- `fill(U,n)` 静态泛型循环填充、单元素 `(T)` cast 读取恢复；
- **接口 clinit**：接口常量初始化（`LIMIT = compute()` 前向方法引用——合法源形）以字段初始化器呈现，`Arrays.asList` varargs 内联，`compute` 内常量折叠（`10*2`→`20`）如实。

## 处置

soundness spec 泛化至三诊断族 + AD scenario；账本 critical 行更新；postfix 恢复片补 AD 锚。

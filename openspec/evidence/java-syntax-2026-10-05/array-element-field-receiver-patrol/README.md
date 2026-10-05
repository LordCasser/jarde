# 数组元素接收者字段读取巡查（2026-10-06 root）——**critical 第 21 锚 / 第 8 族：数组元素接收者类型丢失**

## 主锚（可编译错码）

[fixture/RG.java](fixture/RG.java)：普通类静态查找表 `static{ Item[] all={A,B,C}; for(Item c:all) BY_LABEL.put(c.label,c); }`。
- 渲染：静态块完整幸存（常量构造、Map 分配、数组初始化、for 头），**循环体被引注吞空**——诊断 "the field access at BCI 91 is not one this run proved names the member its own receiver's type declares" + 级联行；
- 剥离（伴生 Item 并入后）编译 exit 0、`-Xverify:all` 运行 **`null/null/null`** vs 原 **`Item@…/Item@…/null`**——查找表静默为空（第一不变量违反；无 enum 常量掩蔽）；
- jadx 完整解（for + `BY_LABEL.put(item.label, item)`）。

## 判别矩阵（全域失败）

| 形状 | 结果 |
|---|---|
| 直接参数/局部别名字段读（`direct(Item)` / `viaLocal`） | **恢复**（`arg0.label`/`local1.label`）——失败面不在"非 this 接收者" |
| for-each 元素字段读 + 累积（RH.total） | 循环体吞空、幸存 `return local1`——渲染 0 vs 原 5（同族） |
| `xs[0].label` 直接（RK/RL `selfElem`/`innerElem`） | 整方法拒（非 void 缺 return=SAFE）——同根因的安全面 |
| 元素先入显式类型局部再读（RK.viaElemLocal） | **仍拒**——呈现写出了 `Item local1 = arg0[0];` 但字段身份证明不用该声明 |
| 当前类数组（RL.selfElem）与伴生类数组（RL.innerElem） | 同败——与组件是否当前类无关 |
| 数组元素**方法调用**（RM：`xs[0].len()` 与 for-each `x.len()`） | **全恢复**（quotes=0 于两方法）——失败面钉死为**字段身份证明独有**，invocation 身份不受 aaload 影响 |
| 元素**字段写**（RN：`xs[0].tag=v` / for-each `x.tag=v`） | 同诊断拒绝——直接形整方法拒、循环形吞空后缺 return=**SAFE**（写侧无幸存可编译错码；读侧 RG 是 critical 面） |

## 归因（初步）

`aaload` 结果值未携带**数组组件类型**：帧/SSA 侧元素值类型缺失，字段身份证明（"receiver's own type declares"）无从成立；呈现层能从上下文写出局部声明，但证明层不用它。组件类型是字节码验证器事实（操作数数组类型的组件），**无需新机制**——应把 aaload 结果类型归到既有类型传播（与 DT-26 的 `array_of_value` 通道相邻但不同：DT-26 是 newarray 值；这里是已有数组引用的元素加载）。

## 处置

**窄片立项 `recover-array-element-field-receiver`**（MVP：aaload 结果携带组件类型；字段身份证明消费它）。enum 静态查找表（EM.Code）为同族数据点——常量渲染本身不可编译（池形名债务）掩蔽了错码，不另计锚。

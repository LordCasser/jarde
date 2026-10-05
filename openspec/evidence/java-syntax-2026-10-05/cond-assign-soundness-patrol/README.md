# 条件位赋值巡查（2026-10-05 root）——critical 第 10 锚 + 两个可恢复性锚

## critical 第 10 锚（copy 族，if 条件位）

`if((v = bs[i]) == true){ any = true; }`——**赋值在 if 条件 + 数组读**：渲染体仅剩循环计数 `local3 = local3 + 1;` 与 `return local1 % 2 != 0;`——隔离编译 exit 0、**`false` vs 原 `true`**（数组读+赋值+比较+any=true 全吞）。诊断双行：数组值无读者（BCI 12）+ **copy 族**（"copy at BCI 13 has no proved local assignment"）。族计数 10 锚/3 族。

## 可恢复性锚（整方法响亮拒绝=安全，但应可恢复）

- **ioLoop**（`while((line = read()) != null)`——**最经典 Java IO 惯用法**）：javap 形 `invokestatic; dup; astore_1; ifnull 16`（**条件位赋值舞蹈：store 与 null 检查双读者**）——诊断 "local 0 crosses a quoted fallback region"。归 [dup-store-conditional 片](../../../changes/recover-dup-store-conditional/)（同类舞蹈不同类型/分支：iadd;dup;istore;ifle → invoke;dup;astore;ifnull）——**引用型/null 分支形锚**。
- **read**（`pos < src.length ? src[pos++] : null`——静态字段后缀作数组下标在三元臂）：javap 形 `dup; iconst_1; iadd; putstatic; aaload`（**静态字段后缀舞蹈**）——诊断 "conditional arm contains an independent instruction"。归 [postfix 片](../../../changes/recover-postfix-old-value-snapshot/)（`arr[idx++]` 消费位变体的**静态字段形 + 三元臂位**）。

## 健康面（负结果）

forAssign（for 头空更新段的赋值在条件外）恢复。

## 处置

soundness spec 泛化已覆盖（copy 族）；两可恢复性锚分别补入 dup-store 与 postfix 片 proposal；账本更新。

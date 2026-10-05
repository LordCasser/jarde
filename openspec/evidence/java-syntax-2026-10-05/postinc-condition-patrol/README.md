# 后缀自增条件位巡查（2026-10-05 root）——可恢复性缺口（SAFE）：经典扫描循环整方法拒

## 发现

三形全部**整方法响亮拒绝**（"local N crosses a quoted fallback region"，body 空 → 编译失败 = **第一不变量不违反**）：
- `do { last = xs[i]; } while (xs[i++] != 0 && i < xs.length)`——**do-while 扫描循环**（C 风格最高频扫描形）
- `while (i < len && xs[i++] != t) { }`——while 复合条件位
- `if (a[i++] > 0 && i < a.length)`——if 短路条件位

**机制**：条件位后缀的旧值 store 使循环局部（i）的定义-使用切片跨循环边/引注区——渲染层选择整方法 explanation-only（与 #66 `i=i++` 的部分吞不同形：循环形状强制切片跨区→过引到整方法）。jadx 全解（`i++` 直接内联进条件）。

## 处置

**postfix-old-value-snapshot 片（恢复方向）补锚**：条件位（do-while/while 复合/if 短路）三形——同为旧值族但现有四形判别未覆盖循环条件位；归同一 SSA 旧值节点机制（呈现层）。不计 critical 锚（SAFE）。

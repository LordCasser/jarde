# 重载解析保真 + 窄型算术巡查（2026-10-05 root，负结果）

## 探针

[fixture/OV.java](fixture/OV.java)（`--release 8`）：五重载族 `f(int)/f(Integer)/f(long)/f(Object)/f(int…)` 的**五个调用点**（精确 int、`(Integer)` cast 强制、`(Object)` cast 强制、多实参 varargs、Integer 参数最精确匹配）+ 窄型算术提升（byte/char/short 的 `a+b` → int → 回 cast）。

## 结果：**健康——重载零漂移的最强证明**（quotes=0 / notrec=0）

- 五调用点逐一精确：`f(1)`（裸 int）、`f((Integer) Integer.valueOf(1))`（cast 强制 Integer 重载）、`f((Object) …)`（cast 强制 Object）、`f(new int[]{1, 2})`（varargs 数组形——物理事实）、`f(arg0)`（Integer 参数精确匹配）；
- **保真证明法**：渲染源经 javac **重新解析**——五重载族在渲染源上重解析出与原完全相同的重载选择（`intIntegerObjectvarargsInteger` 串逐字符一致）；若任何调用点的 cast/装箱呈现漂移，重载串必然变化——这是比逐点比更强的整体不变量验证；
- 窄型提升 `(byte)(arg0 + arg1)` 三型（byte/char/short）全对（-56 回绕与 char 和一致）。

## 处置

负结果归档，不立 spec。重载保真（五重载×五形）与窄型提升确认覆盖。

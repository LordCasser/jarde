# 装箱类型到 Number 平台上转型巡查（2026-10-03）

[复合窄化/类型见证巡查](../../../../evidence/java-syntax-2026-10-03) 的附带发现（主线 `c6b64900`）。同域其余形态**全部健康**：复合赋值隐式窄化（`(byte)(x op n)` 显式 cast 形态、行为一致）、char 算术、接口常量（javac 内联 + `static interface` 折叠）、显式类型见证调用（`this.<String>pick` 位置无痕迹）、装箱三元、循环携带 StringBuilder。固定转录 [fixture](fixture/)（C7 健康对照 + C8 缺口；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）。

## 表现

`static <T extends Number> T larger(T a, T b)` 被 `larger(3, 7)` 调用：int 字面量装箱为 `Integer` 实参、形参擦除为 `java.lang.Number`——**`Integer → Number` 上转型无证据通道**：`the parameter 0 … declared java.lang.Number presents java.lang.Integer but … no safe reference conversion evidence` → 语句被引、重编丢行（输出缺 `7`）。

## 根因

`platform_reference_argument_widens` 闭集仅覆盖 `List→Iterable` 与 java.util 集合 40 对；**java.lang 装箱类→其实现接口/父类**（Integer/Double/Long/Float/Short/Byte/Character/Boolean→`Number`〔Character/Boolean 除外〕与全部→`Comparable`、String→`Comparable<String>`/`CharSequence`）未入表。`<T extends Number>` 泛型方法以字面量/装箱值调用是标准库与业务代码高频形态。

## 处置方向

`recover-boxed-number-widening`（窄切片）：java.lang 闭集直接边入表（8 装箱类→`Number`〔六数值型〕、→`Comparable`、`String→CharSequence/Comparable`），walk/核对同 throwable/collection 两先例；C8.main 恢复（`larger(3, 7)` 呈现）且行为逐字一致（`x`/`1:2`/`7`/`eoeoeoe`）；两既有闭集与用户类负例零回退。

原 class 为行为基准。

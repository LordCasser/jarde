# 构造委派链 + 丢弃分配巡查（2026-10-05 root）

## 健康面（负结果——经典难点全过）

[fixture/CD.java](fixture/CD.java)（`--release 8`）——**委派链与字段初始化序全部忠实**：
- 三级继承 `C→B→A`，`C(){ this(30); … }` → `C(int){ super(40); fc=init(…); … }` → `B(int){ super(21); fb=init(…); … }` → `A(int){ super(); fa=init(…); … }`——**this()/super() 带实参委派 + 字段初始化按 JLS 序并入委派目标构造器**全部如实（含 A 无参 `this(10)` 再委派）；
- `D(){ this(7); fd += 1; }`——委派后语句保序；
- 副作用方法 `init()` 的调用序呈现正确。

## 发现：丢弃分配语句（响亮拒绝，第 5 个新证缺口）

[fixture/DN.java](fixture/DN.java) 判别实验——**唯一变量是分配结果是否被消费**：

| 形 | 结果 |
| --- | --- |
| `discarded(){ new N(); println("after"); }`（结果丢弃） | **拒**（3 引注：BCI 0/3 分配+copy 无形状、copy 无局部赋值） |
| `used(){ N n = new N(); n.hi(); }` | 恢复 |
| `chained(){ new N().hi(); }` | 恢复 |
| `argUse(){ takes(new N()); }` | 恢复 |

CD 的 `main` 中 `new C();`（副作用构造、结果丢弃——真实常见形：自注册 Timer/Listener）同因拒绝。**jadx 完整恢复**（[results/jadx-DN.java](results/jadx-DN.java)：`new DN.N();` 语句呈现）。

## 处置

第 5 个新证窄缺口（呈现域：结果丢弃的分配语句——`new N();` 作为独立语句呈现；构造器副作用序保留）。登记 summary + 随后立项窄片。

# 链式赋值与副作用下标巡查（2026-10-05 root）

## 健康面

`x = y = 7`（**局部链**）恢复为拆双赋值（`int local1 = 7; int local0 = local1;`）。

## 发现一：链式字段赋值（第 11 个新证缺口）

`CH.a = CH.b = CH.c = 5` 拒（javac 发射 `iconst_5; dup; putfield c; putfield b; putfield a`——dup 值跨 putfield 存活）。**jadx 有解**：三个独立赋值（`c=5; b=5; a=5;`——从右到左求值序保留为语句序）。局部链（dup 跨 istore）已恢复——判别=dup 的存储目标是字段（putfield）还是局部（istore）。

## 发现二：副作用下标（后缀片的消费位变体）

`arr[idx++] = 10` / `arr[idx--]` 读位拒——旧值快照（load-before-iinc）被消费于**数组下标位**（"copy … has no proved local assignment"）。jadx 以展开 temp 形解（`int i = idx; idx = i+1; iArr[i] = 10;`）。归 [#9 后缀旧值片](../../../changes/recover-postfix-old-value-snapshot/)的消费位扩展锚（呈现可取 jadx 的 temp 形而非内联 `i++`）。

## 处置

链式字段=第 11 窄片立项；副作用下标=#9 片变体锚（消费位扩至下标位、呈现允许 temp 形）。

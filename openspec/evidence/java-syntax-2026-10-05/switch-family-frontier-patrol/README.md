# switch 家族前沿巡查（2026-10-05 root，负结果）

## 探针

[fixture/SW.java](fixture/SW.java)（`--release 8`）：return 形、**共享尾变量形**（case 内赋值+break、switch 后 return——最常被反编译器做成 goto 的形）、String switch（含 `case "b": case "c":` 合并）、合并 int case + case 块内声明局部、default 无 break。

## 结果：**健康，无缺口**

源码区 quotes=0 / notrec=0，全部形呈现为结构化 switch（无 goto）：

- **共享尾变量**：`t` 呈现为 `case N: local1 = X; break; … return local1;`——与 jadx 同构（jadx 也是赋值+break 形）；
- String switch 合并 case（`case "b":` `case "c":` 空落）保持；
- int 合并 case + case 块内 `int local1 = arg0 * 2;` 声明保持；
- **行为**：渲染源集 `javac` exit 0，`java -Xverify:all` 输出 `one/20/BC/6` 与原 class 逐行一致。

## 处置

负结果归档，不立 spec。switch 家族（含此前 enum-switch-labels 域外的 String/合并/块作用域变体）确认覆盖。

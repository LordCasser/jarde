# this 限定与跨类成员访问巡查（2026-10-05 root，负结果）

## 探针

[fixture/OQ.java](fixture/OQ.java)（`--release 8`）：内部类 `Outer.this.method()` 显式限定调用、裸隐式调用、`Outer.this.field` 字段访问（经 `access$000` 读访问器——**EM-15 域的读形已恢复**）、返回外类实例、静态嵌套访问外类静态/经参数访问外类实例成员。

## 结果：**健康，无缺口**（宿主 + 伴生 quotes=0）

- **`OQ.this.get()`** 显式限定与**裸 `get()`**（viaBare）都呈现为 `OQ.this.get()`——裸形统一到限定形（等价规范化，语义精确）；
- **`OQ.this.v` 字段访问**呈现为 `OQ.access$000(OQ.this)`——私有字段经读访问器（物理事实如实；读形访问器已由既有域恢复）；
- **`OQ.this` 返回**、**静态嵌套 `OQ.sget()`**、**经参数 `arg1.get()`** 全部恢复；
- 拼接三类型 `javac` exit 0、`java -Xverify:all` 输出 `7/7/7/7/8/7` 与原 class **逐行一致**。

## 处置

负结果归档，不立 spec。this 限定族（方法/字段/返回/静态嵌套双向）确认覆盖。

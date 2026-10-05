# 初始化器前向引用（经方法）与 clinit 序巡查（2026-10-05 root，负结果）

## 探针

[fixture/FR.java](fixture/FR.java)（`--release 8`）：**经方法的前向引用**（`early = initEarly()` 读 `LATER`——直接前向非法、经方法合法）、静态/实例初始化器源序、编译期常量内联（`usesC` → `6`）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）——clinit 序保真

- **clinit 源序逐字保留**（`FR.early = initEarly(); FR.LATER = 100;`——**early=2 的可观察 quirk 由序保证**：initEarly 运行时 LATER 仍是默认 0）；实例初始化同形（`ix = iInit(); iy = 30;`）；
- 常量内联（`C` 用点呈现 `6`——javac 常量折叠忠实）；`initEarly` 内 `FR.LATER + 2` 限定引用恢复；
- 行为 `2/1/6/100` 逐行 IDENTICAL——**重编译后 quirk 复现**（这是 clinit 序保真的最强验证：渲染源的初始化顺序重演出相同的中间态）。

## 处置

负结果归档，不立 spec。初始化序族（前向经方法/静态实例双形/常量内联）确认覆盖。

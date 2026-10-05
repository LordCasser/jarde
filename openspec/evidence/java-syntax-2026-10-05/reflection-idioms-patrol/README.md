# 反射惯用法族巡查（2026-10-05 root）——主体健康；kind 分支域数据点

## 探针

[fixture/RF.java](fixture/RF.java)（`--release 8`）：`Class.forName().newInstance()`、`getClass().getMethod(m,String.class).invoke(target,arg)` 反射链、`isInstance + cast` 守卫、`getConstructor(String.class).newInstance(arg)` + `(String)` checkcast 消费——插件式/ServiceLoader 风格代码骨干。

## 结果：**3/4 完整恢复 + 1 SAFE 分支引注**（无新族）

- **load/call/ctor 全恢复**（重头反射链）：`Class.forName(arg).newInstance()` 直链、`saved0 = arg0.getClass()` 复用局部 + `.getMethod(...).invoke(...)` 深链、`getConstructor + newInstance + (String)` checkcast 消费——`Class` 值以 saved0/local 命名如实呈现；
- **kind = 分支体引注（SAFE）**：if-真分支体（`c.cast(o).toString().length() > 0` cast 链）整分支 `@bytecode 32` 引注（"live block reachable only through edges the normal-flow view leaves out"）——剥离后**缺 return 语句 → 编译失败 = 响亮拒绝**，非 compilable-wrong；归既有分支域引注模式（FO 族同域），非新族；
- 行为隔离验证：load/call/ctor `true/abc/made` 一致（kind 以与实际返回一致的 stub 验证 main）。

## 处置

负结果归档，不立 spec。反射域（forName/newInstance/getMethod-invoke/getConstructor/isInstance/cast）确认覆盖；分支域 cast 链引注为既有模式数据点。

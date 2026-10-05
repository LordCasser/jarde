# 显式锁族巡查（2026-10-05 root）——三形全拒；put() = void 洞活例

## 发现

ReentrantLock + Condition 真实并发骨架（put/take 有界缓冲 + tryLock 守卫）三形全拒：
- **put()（void）触发既有 whole-method-quote 守卫**："the quoted block … ends in a control-flow exit … the body without it would still compile and silently change what the method does; the whole method is quoted"——守卫自己承认风险但 void 方法退化后是**空 void 体 = 剥离注释可编译静默 no-op（void 洞活例，当前主线就有）**；这正是 soundness 片 void 裁决（`jarde_refused_body()`）要收口的形状——**收编为该片验收 fixture**（守卫落地后 put() 剥离 SHALL 编译失败）；
- take()（int，return 在 try 内）/ tryLockQuick()（boolean，tryLock 守卫+finally 复制）：引注诊断（exception-handler shape / finally-copy merge 未证）→ 空/残 body 缺 return = **SAFE**；
- jadx 全解（lock; while+try/catch-await; finally-unlock 复制形）。

## 处置

**显式锁域 = local-scope 第 12 锚**（lock/unlock try-finally + await 条件队列——j.u.c 最高频骨架）；put() 作为 soundness 片 void 修复验收 fixture 记入其 proposal（不另立）。

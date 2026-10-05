# wait/notify 并发族巡查（2026-10-05 root，负结果——经典并发惯用法全过）

## 探针

[fixture/WN.java](fixture/WN.java)（`--release 8`）：ACC_SYNCHRONIZED 方法（`synchronized void set`）、**wait 条件循环**（`while(!ready){ wait(); }`——经典 guarded wait）、同步块（显式 lock 对象与 `synchronized(this)` 双形）、`notifyAll`、`throws InterruptedException` 链。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **同步方法**：ACC_SYNCHRONIZED 标志正确呈现为 `synchronized` 修饰符；
- **wait 循环**：条件循环 + `this.wait()` 恢复；同步块的 **monitorenter/exit** 折回 `synchronized (lock) {…}`（显式对象与 this 双形）；`notifyAll`/`wait` 的 receiver 限定如实；throws 链呈现；
- 行为 `42/7/8` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。并发原语族（monitor/wait/notify/双同步形）确认覆盖。

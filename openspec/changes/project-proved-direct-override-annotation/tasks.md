## 1. 准确父方法证明

- [x] 1.1 在既有类源码请求内选择同 jar 唯一直接普通父定义及完整方法表，以父/子物理身份、同包、准确名称和描述符证明可覆写实例方法；用同包正例及 private、跨包包私有、static/final、缺失/歧义父定义负例验证。
- [x] 1.2 对桥/合成、Signature、重复或截断成员、错误 flags、预算和取消保留无注解或既有请求停止；用定向测试确认没有凭名称碰撞产生 `@Override`，且不发布半份源码。

## 2. 最终类源码投影

- [x] 2.1 只给获证子方法最终文本加一次 `@Override`，保留物理方法/注解事实和正文来源；用报告断言及 Java 8 重编确认声明、body 未被改写。
- [x] 2.2 执行 [EM-02 冻结 replay](../../evidence/java-syntax-2026-09-27/em02-modifiers/replay.py) 的原/JADX/Jarde 六类完整源码重编、`java -Xverify:all` 和三方输出对照，确认同包正例及两个同名非覆写负例；运行适用 Rust 回归、workspace check、格式检查和 `openspec validate project-proved-direct-override-annotation --strict`，记录验收与剩余边界。

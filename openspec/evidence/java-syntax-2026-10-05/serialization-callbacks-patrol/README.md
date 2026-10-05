# 序列化回调族巡查（2026-10-05 root）——回调域健康；main=第 6 族簇新位点

## 探针

[fixture/SZ.java](fixture/SZ.java)（`--release 8`）：`serialVersionUID`、transient 字段、**JVM 反射回调私有方法**（`writeObject`/`readObject` 含 `defaultWriteObject`/`readInt`）、`writeReplace`、main 内完整序列化往返。

## 结果

- **序列化回调域健康**：`writeObject(ObjectOutputStream)`/`readObject(ObjectInputStream)`（throws IOException+ClassNotFoundException 双异常子句）、`writeReplace()`（throws ObjectStreamException）、`transient int cache` 修饰符+字段初始化序全部恢复（quotes=0 于这些成员）；
- **main = 已知族级联（SAFE）**：15 quotes 全在 main——
  - **第 6 族新位点 ×2**：`new ObjectOutputStream(bos)`（`ByteArrayOutputStream → OutputStream` 类→接口宽化）与 `new ObjectInputStream(...)`（`ByteArrayInputStream → InputStream`）——**平台宽化 × JDK ctor 实参位**（与 regex-matcher 的宽化×方法参数位配对新组合）；序列化往返是最高频样板代码之一，位表价值高；
  - P3 2b.2 级联行（local3/local4 读被拒写者——soundness 片枚举的级联行原样出现）；
  - 多消费者 3 consumers + no-bounded-final-consumer ×7（第 4 族级联行）。
- 剥离后 main 因 local3 未声明不可编译 = 级联 SAFE；回调成员独立完好。

## 处置

回调域不立项。**第 6 族簇候选窄片补位点**：ObjectOutputStream/ObjectInputStream ctor 实参宽化（平台宽化×ctor 位）入合并窄片位表（序列化样板最高频消费方）。

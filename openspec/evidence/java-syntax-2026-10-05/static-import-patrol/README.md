# 静态导入与跨类静态引用巡查（2026-10-05 root，负结果）

## 探针

[fixture/Lib.java](fixture/Lib.java)+[SI2.java](fixture/SI2.java)（`--release 8`）：`import static` 编译后**消失**（字节码只剩限定引用）——探针以等价的直接限定形（`Lib.SV`/`Lib.add(1,2)`/`Lib.Nested.NS`——**与静态导入编译产物字节码同形**）验证跨类静态解析。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **跨类静态字段/方法/嵌套类静态字段**三类限定引用（`Lib.SV`、`Lib.add(1, 2)`、`Lib.Nested.NS`）全部恢复——外部类限定名**逐字保留**（无短名化）；
- **往返验证**：渲染源 + 真实 `Lib.class` 同 classpath 重编译 exit 0、行为 `42/3/7` 与原 class **逐行一致**——跨类解析在渲染源重编译下正确闭合（编译器按限定名找到外部类）；
- 静态导入的"源形恢复"（重写 `import static` + 短名）非字节码可证信息（导入已消失）——**不恢复是忠实**，限定形即正确呈现。

## 处置

负结果归档，不立 spec。跨类静态引用（static import 的编译后形态）确认覆盖。

## 1. 冻结形状与三方基线

- [ ] 1.1 自写两资源 Java 8 fixture（正例：读取返回/内层抛出/外层 close 抛出/正常关闭四条路径；负例：patch 行范围错一级），javac --release 8 与 9+ 各编译一份，冻结 SHA；记录 javap 行几何、原 class 执行输出（含 suppressed 顺序）、jadx 1.5.6 输出（展开为嵌套 try，记为其偏离）、jarde 修前输出（RangeEnd 引用）。

## 2. 层级几何与呈现

- [ ] 2.1 `twr` 几何校验改为「每层行止于该层正常 close 链起点」；单资源路径回归保持绿。定向测试：两层链通过、patched 负例拒绝保持、单资源不变。
- [ ] 2.2 多资源一条头的呈现：资源按初始化顺序、close 逆序证明逐层复用、suppressed 链锚点保留；文本含两个资源声明与体语句，无 `@bytecode`。

## 3. 对照与门禁

- [ ] 3.1 恢复文本 Java 8 重编译，四条路径执行对照与原 class 一致（返回值、异常类型、suppressed 顺序）；jadx 偏离写入对照记录。
- [ ] 3.2 复跑 guard、twr、typed_catch、execution_comparison 回归；`cargo fmt`、`cargo clippy -p jarde-java -p jarde --all-targets -- -D warnings`、`openspec validate recover-multi-resource-twr --strict`；golden/语料计数若变重录并说明。

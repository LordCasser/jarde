# 泛型投影的物理名称反例

五个受控 javac23 --release8 -g:none class，来自 OpenSpec `s1-generic-physical-names-v1/matrix-inputs-r1` 的 supplementary 输入。只嵌入目标 class，不含 driver；源码、编译器命令和原始反射 oracle 由该检查点固定。覆盖 field 名、两种 constructor owner、私有泛型调用名和调用 owner。

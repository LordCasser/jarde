# Nested saved-return finally

完整目标类只有构造器和与封存EX同体的nestedFin；独立driver覆盖18种返回/算术异常/两层清理异常组合。

v8由OpenJDK23.0.1 javac --release 8编译，v8-javac8由Corretto1.8.0_432编译，两者各509B、Code72B/4条catch-all/35指令。实际编译及原class运行收据见OpenSpec的baseline-matrix-root-v2和sealed-class-oracle-root-v1；不得修改生成源码制造通过。

- `source/NestedFinally.java`：SHA256 `82e94215125e9454bbb16cf6740751463d071ba455690bd27be4f425c8ceff67`
- `source/NestedFinallyDriver.java`：SHA256 `0f943469493a91928d9cfd727abffe1b1f12333b9b301b76386e507fa71ac807`
- `v8/NestedFinally.class`：SHA256 `3e4173c8e371ed5aae4e451122506dad3fa4d872485fe8e22c71e88a8c7f9890`
- `v8-javac8/NestedFinally.class`：SHA256 `cc9bc23f75d9b5b61bb813660cd9e6566e5363f3dd6a44cebe810921423b346e`

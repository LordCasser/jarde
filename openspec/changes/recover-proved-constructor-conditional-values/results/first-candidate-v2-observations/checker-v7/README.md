# Constructor/conditional checker v7 private candidate

This candidate derives from checker v6 and changes Javap method lookup only. The shared `javap_method_block` selector recognizes `<clinit>()V` only when its declaration is exactly `static {};`; both instruction-BCI and exception-table parsing use this selector. The selected initializer still flows through the existing physical method identity/order, owner, flags, Code envelope, BCI, catch-table and source-map checks. No initializer is skipped.

The frozen input is `/private/tmp/jarde-constructor-conditional-fixed-candidate-root-v2/execution.json`. Exact base/candidate/capture/parser/Javap/class pins are in `pins.json`, and `check_controls.patch` is the v6-to-v7 diff. Checker and toolchain were not run during preparation.

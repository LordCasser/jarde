# String switch with a middle default label

This Java 8 fixture adds a shape not present in the earlier string-switch variants: `default` appears between ordinary labels, executes its own statement, and falls through into the following empty-string label. It also covers the `Aa`/`BB` hash collision, another explicit fallthrough, Unicode, a single selector evaluation, null, and a throwing selector.

The frozen source and runner are compiled with:

```sh
javac --release 8 -g:none -d v8 StringSwitchMiddleDefault.java StringSwitchMiddleDefaultRunner.java
```

Frozen class hashes:

- `StringSwitchMiddleDefault.class`: `fd6caaaac6c00799e218601c76a6dc133238a851d05672237219bfbebfb2a647`
- `StringSwitchMiddleDefaultRunner.class`: `d624a2eb51d4d302f2bd8ae503373a3415dab0dc8a1cc4e3d2343c088aee9d1d`

# Local-declaration initializer site — shared-scan containment hold

This fixture freezes design criterion 5's containment negative (`recover-anonymous-local-decl-site`):
`main` holds one proved anonymous **interface** allocation at a local-declaration initializer
position, mid-method, with statements after it. The site scan is shared by the anonymous interface
projection and the anonymous superclass projection, so before the explicit shape gate this form
would have activated the interface path with no evidence of its own.

The frozen `.class` files were produced with:

```sh
javac --release 8 -g:none -d . LocalDeclInterfaceHold.java
```

`LocalDeclInterfaceHold.root.rendered.txt` is the root class's byte-exact class-source rendering.
It is identical before and after the slice (SHA-256 `1badfcb5b9dcb9a46bf017e3b285073e8424c8143a239f3a6bac9606efc98ce5`
on both legs); the projection state moves `absent` → `refused` with reason
`anonymous_interface_site_shape_unsupported`, which is the diagnostic-level record of the explicit
containment. Original class behavior under `java -Xverify:all`:
`after-allocation|label:inside|run:inside`.

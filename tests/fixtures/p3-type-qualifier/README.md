# Type qualifier binding fixture

`TypeQualifierProbe.class` is the only permanent class.  `arg0`, `arg0_2`, `ShadowOther`, and the
runner are source-only JDK inputs.  The target deliberately keeps an unused `ShadowOther` parameter
while its body uses default-package owners `arg0` and `arg0_2`; the recovered names therefore must
avoid both owner prefixes.  `ownCall` is the no-conflict same-class static-call control.

The runner resets both external static fields before each null/non-null case and records the own
call, static call, field read, and write targets plus the instance field.  The debug `java` package
prefix and method-reference qualifier cases remain source-only evidence under the change fixture
notes; they are not folded into this positive class.

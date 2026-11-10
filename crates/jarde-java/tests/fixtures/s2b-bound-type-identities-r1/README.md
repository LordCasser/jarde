# S2b bound type identity fixtures

This fixture adds real classfile controls for two distinct physical definitions with binary name `identity/Foo`, a three-level declared generic hierarchy, and the exact Java 8 `String.class` header. The existing 13 signature-path classes are copied byte-for-byte from `s2b-signature-paths-r1`; `String.class` is copied as a single entry from the pinned Corretto 8 `rt.jar`.

The two `identity/Foo` definitions were separately compiled with Corretto 8 using `-source 8 -target 8 -g`. The generic chain was compiled against only the fixture's `identity-a/classes` plus the JDK boot classes. Source files, compiler path/version, commands, and SHA-256 pins are recorded in `manifest.json`.

Intended loader controls: present `identity-a/classes` and `identity-b/classes` as distinct roots/loaders and verify that selecting the same binary name preserves distinct physical definitions. The generic chain checks binding of `Base<T>`, `Mid<X> extends Base<List<X>>`, and `Child extends Mid<identity.Foo>`. The platform addition is only `java/lang/String.class`; no recursive platform closure was copied.

No Jarde command, Cargo command, or target execution was run. The fixture does not include holdout bytes.

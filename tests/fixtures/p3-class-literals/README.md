# Class literal recovery fixtures

This directory freezes the primary Java 8 source/runner pair and the small type-name boundary
inputs for `openspec/changes/recover-class-literals/`. The class bytes were compiled with Corretto
8 (`javac -source 8 -target 8 -g:none`) into separate output directories; separating them matters
because the top-level type named `java` in the collision input would otherwise shadow the `java`
package while compiling the local-name control.

| Class file | Source | SHA-256 | Purpose |
| --- | --- | --- | --- |
| `v8/ClassLiteralProbe.class` | `ClassLiteralProbe.java` | `11b1724cd33490629a115dfdd8104fed394f3569b03aa6ecfe9007ca1bedfc4b` | Frozen 851-byte evidence class: reference, reference-array, primitive-array, self, primitive/void `TYPE`, and one call argument. |
| `v8/LocalNameQualifier.class` | `LocalNameQualifier.java` | `446bc3b7c00c5297dd11ed3aa323839529cf8adfa17fd2230217732b43e2d01f` | A local named `java` plus a stored class literal; the local does not shadow a type-context package name. |
| `v8/String.class` | `TypeNameString.java` | `4b26234a1ba1c8c3316c229c89dd1db9dde1ebb27e468030bbfda117a5103e41` | Current class name `String`; `java.lang.String.class` remains safely spellable because `java` is the path's first component. |
| `v8/java.class` | `TypeNameCollision.java` | `903688a81233318a964322fe135b3cc0f269e7015f16c477dbf3df78327ac8e4` | A current type named `java` whose source returns `String.class`; emitting the pool name as `java.lang.String.class` is unsafe, so single-class recovery quotes the producer and consumer. |

The frozen primary class matches the evidence source at
`openspec/evidence/java-syntax-2026-09-22/class-literals/`. Its `javap.txt` records the five
`ldc` sites at BCI 0 and pool indices 13, 15, 17, 8 and 13 respectively; the call at BCI 2 uses
pool index 28. The seven runner lines are frozen in
`openspec/evidence/java-syntax-2026-09-22/class-literals/original-run.txt`.

The type-name collision input intentionally freezes only `java.class`; no other type is needed to
prove the current declaration's unsafe `java.lang` root. The nested-type redirection probe is
recorded in the OpenSpec analysis. Unicode MUTF-8 decoding is tested at the decoder boundary;
non-ASCII Java identifiers, including the Corretto 8-valid U+10400 name, remain an explicit
source-spelling refusal until Java 8's Unicode 6.2 identifier tables are handled as a separate
scope. Binary names containing `$` are also refused because a single-class pool cannot distinguish
a nested class from a top-level `$` name or verify the source/classpath spelling needed for it.

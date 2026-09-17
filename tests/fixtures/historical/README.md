# Historical classfile corpus

This directory contains a small, reproducible corpus for P0 classfile inspection. The
fixture source is `ecj-4.6.1/src/HistoricalControlFlow.java`; each `vNN/` directory
contains the corresponding `HistoricalControlFlow.class`.

## Provenance

- **Compiler:** Eclipse Compiler for Java (ECJ) 4.6.1, Maven Central URL
  <https://repo1.maven.org/maven2/org/eclipse/jdt/core/compiler/ecj/4.6.1/ecj-4.6.1.jar>.
  SHA-256: `9cddda75f4a1b4469e73f44e7b61a3e897d0f657df4797f9106ffe88c4eeade0`.
  `java -jar ecj-4.6.1.jar -version` reported
  `Eclipse Compiler for Java(TM) v20160829-0950, 3.12.1`.
- **Bootclasspath:** Temurin JRE 8 `OpenJDK8U-jre_x64_linux_hotspot_8u504b01.tar.gz`,
  URL
  <https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u504-b01/OpenJDK8U-jre_x64_linux_hotspot_8u504b01.tar.gz>,
  published archive SHA-256:
  `52dcd578baca1d3e449ea86768a9129c0ee04d7b22565695498353cc66940c61`.
  The extracted `lib/rt.jar` SHA-256 is
  `ebcc5ee761cb12c3f37222da14bdf14e4e9f16e86b60c21827e6252e38c91531`.
  It supplied platform symbols only; it did not determine output code generation.
- The compiler and JRE archive are generation-only inputs. They are not checked into
  this repository and are not runtime or test-time dependencies.

The source was compiled with `-g:none` and the fixed JRE 8 bootclasspath using these
exact command forms (with `<rt.jar>` and `<out>` expanded per output):

```text
java -jar /tmp/ecj-4.6.1.jar -source 1.3 -target 1.1 -g:none -bootclasspath <rt.jar> -d <out> HistoricalControlFlow.java
java -jar /tmp/ecj-4.6.1.jar -source 1.3 -target 1.2 -g:none -bootclasspath <rt.jar> -d <out> HistoricalControlFlow.java
java -jar /tmp/ecj-4.6.1.jar -source 1.3 -target 1.3 -g:none -bootclasspath <rt.jar> -d <out> HistoricalControlFlow.java
java -jar /tmp/ecj-4.6.1.jar -source 1.3 -target 1.4 -g:none -bootclasspath <rt.jar> -d <out> HistoricalControlFlow.java
java -jar /tmp/ecj-4.6.1.jar -source 1.5 -target 1.5 -g:none -bootclasspath <rt.jar> -d <out> HistoricalControlFlow.java
java -jar /tmp/ecj-4.6.1.jar -source 1.6 -target 1.6 -g:none -bootclasspath <rt.jar> -d <out> HistoricalControlFlow.java
java -jar /tmp/ecj-4.6.1.jar -source 1.7 -target 1.7 -g:none -bootclasspath <rt.jar> -d <out> HistoricalControlFlow.java
java -jar /tmp/ecj-4.6.1.jar -source 1.8 -target 1.8 -g:none -bootclasspath <rt.jar> -d <out> HistoricalControlFlow.java
```

## Outputs and expected capability

All outputs are class `HistoricalControlFlow`; the 45.x output has historical minor 3 and
the 46–52 outputs have minor 0. Their classfile versions, sizes, and SHA-256 values are:

| directory | version | bytes | SHA-256 |
| --- | ---: | ---: | --- |
| `v45` | 45.3 | 258 | `ecd9c7cca0cd3c145be129e3342d4af6e985a2872091cc2b848e557370f5438d` |
| `v46` | 46.0 | 258 | `f86c96249ff7bb22f5ca3fa2ae03b39c13c403161011535e9a96c31ed8bac590` |
| `v47` | 47.0 | 258 | `e8bfc5ddc2317674734735a3d62e3f0f5f9d95592696da76b724e293216160e9` |
| `v48` | 48.0 | 258 | `f2ce3395e17a5d86f90b9980ded6bad5aff6d0e8e65c7851f010e7ee1a3e03d7` |
| `v49` | 49.0 | 250 | `5d4ee680c482b586af56347583ea03b4bd3215c0736d461be6ad79765765df0c` |
| `v50` | 50.0 | 303 | `75ec3c5568c4797002707e5c350550de00292cca3ea00a0afec471d528c103a2` |
| `v51` | 51.0 | 303 | `bd71cd1ffaedba19ba6ac90d88c06025f466e804db501b0bb3c5558eebae1fd6` |
| `v52` | 52.0 | 303 | `f9b6566fc4533e3be181ef7bf1455d478fb016f0d9a5e1f685a34901ddb0b6ad` |

These are the Java 8 runtime profile's accepted classfile versions for the P0
version-only gate. The test uses public `inspect_header` in `Strict` mode and expects a
complete structural read with `VerificationStatus::NotPerformed`. It then uses public
`inspect_method_bytecode` on `finallyPath(I)I` and checks complete instruction-boundary
coverage, not JVM verification.

The 45.3--48.0 outputs contain legacy `jsr` at BCI 5 and 12 and `ret` at BCI 21.
The 49.0--52.0 outputs inline the `finally` path and contain neither opcode. This is
historical ECJ 2016 output behavior observed in this source slice. It is not a set of
hand-crafted boundary fixtures, and it must not be read as output from a compiler from
each original classfile-release era: one 2016 ECJ compiler generated all eight targets.

The source is project-owned fixture input. The compiler is EPL-licensed; that fact is
recorded only as provenance, without making broader licensing conclusions about the
project's compiled fixture outputs.

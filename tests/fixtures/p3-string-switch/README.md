# String switch proof fixtures

`v8/StringSwitchProbe.class` is the 635-byte Java 8 class from
`openspec/evidence/java-syntax-2026-09-22/string-switch/StringSwitchProbe.java`.
Build with `javac --release 8 -g:none`; SHA-256:
`258fea472bfd8f80ee49726dc25f0c3d46dce8f61db57d43d5f63e52e6c2c53c`.
It contains the `"Aa"`/`"BB"` hash collision and a second discriminator switch.

`ExtraBucketEffect.java` is a legal two-switch shape with an observable `calls++`
inside one hash bucket. Build with `javac --release 8 -g:none`; the resulting
`v8/ExtraBucketEffect.class` SHA-256 is
`df7bf504539b2d52a7029101ce2a16961c2771c7a7a7c645f366f937c4bd09e2`.
The local proof must refuse it because the bucket has an effect outside its
`equals`/constant-discriminator subgraph.

# enum-string-field-name — the non-`op` control probe for the DT-12 String-argument slice

`recover-enum-string-field-name-generalization` (2026-10-04). The accepted
`recover-proved-string-arg-enum-constant-bodies` slice (DT-12) hard-coded the String source
argument's field name to `op`, which is exactly the frozen anchor `DoubleOperations`' field name —
so the acceptance could not see the hard-coding. This fixture is the same two-constant
anonymous-body enum shape with the field named `label` (and getter `getLabel`), so an
implementation that hard-codes `op` fails it and an implementation that reads the name from the
constructor bytecode projects it.

## Source

- `LabeledOps.java` — `demo.LabeledOps implements LabeledValue`, constants `TIMES("*")` and
  `DIVIDE("/")`, each with its own `apply` body; `private final String label;` assigned by
  `LabeledOps(String label)`.
- `LabeledValue.java` — the one abstract method the bodies implement (`double apply(double,
  double)`).
- `Probe.java` — prints one event line per constant (`name=label:ordinal:apply:class`); the
  behavior baseline for recompile-and-run comparisons.

## Compiled classes

`demo/*.class` are the `javac --release 8 -g:none` outputs of the two enum sources, frozen so the
projection test reads the exact bytes:

```
javac --release 8 -g:none -d classes demo sources...
cd classes && jar cf ../labeled.jar demo
```

`Probe.class` is the same release compiled from `Probe.java` for the original-run leg.

`original-behavior.txt` is `java -Xverify:all -cp classes demo.Probe` over these bytes:

```
TIMES=*:0:6.0:demo.LabeledOps$1
DIVIDE=/:1:2.0:demo.LabeledOps$2
```

## Guard

`tests/enum_string_field_name.rs` reads these bytes (`include_bytes!`), packs the stored zip,
requests `demo.LabeledOps`, and asserts the constant-list projection with
`this.label = arg0;` spelled from the proved field name. On the pre-fix baseline the same bytes
degrade to `public static final demo.LabeledOps TIMES;` (illegal Java; `javac` exit 1
`此处需要枚举常量`) — that refusal shape is what this fixture exists to catch.

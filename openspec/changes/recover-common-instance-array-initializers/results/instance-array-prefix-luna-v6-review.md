# Instance-array initializer private V6 review

V6 is an unapplied replacement for the V5 private patch plus one permanent facade-level positive using the exact frozen javac8 `CommonNoClinitArrayInit.class` bytes.

Patch SHA-256: `391a5e9bbb863a5db09ee9caac0ace64aa2f4d556c560605b00804cf34894dda`

Fixture input:

- `no-clinit-super-args-v1/baseline-root-v2/cases/javac8-original/classes/CommonNoClinitArrayInit.class`
- SHA-256: `0249e233851cadf92ff7f160b97c56b98521e57e327b5d430306029b89f598e4`
- The class declares `first:[B` and `second:[B` at physical field indices 1 and 2, has two direct-super constructors, and declares no `<clinit>`. The source baseline and runtime were independently accepted; this test consumes the frozen class bytes, not the source as candidate evidence.

The added test reuses V5's existing `source_for` and one-class archive helper. The parent class bytes are unnecessary for this assertion: the candidate is scoped to this physical class's own fields and constructor prologues; the typed superclass name and each `super(...)` expression are already present in the class and method facts. It does not analyze or project the parent's constructor. If the facade unexpectedly requires resolving that parent, the test will fail and can be revised to use the existing ZIP writer with the frozen Base class.

The test asserts both array declarations are present and remain in physical field order, no `<clinit>` is required, and both physical constructors remain reported with their distinct `super(7)` / `super(arg1)` calls, `mark(11)`, `mark(12)`, `run(21)`, and the trailing `trace` updates (`+ 91` / `+ arg1`). It also checks each constructor's original two `putfield` source-map BCIs (26/41 and 25/40) still point into its physical recovery text, while the assembled source contains the field initializers. This guards the class-level projection from replacing physical method evidence.

The class-source API exercised by this existing facade test helper has no separate `default` versus `all` selector. It requests the complete snapshot class view and exposes every physical method; the independent V2 baseline already established byte-identical default/all assembled text. The new assertion checks that all four declared physical methods are present through the report and that both constructors are complete. It does not claim a new two-mode comparison.

## Validation boundary

Only this new patch and review were written. The V5 patch and the frozen source/class/baseline evidence were left unchanged. No product source or test file was edited, and no Git, Cargo, rustfmt, JDK, JADX, or Jarde command was run. The patch hunk counts were recalculated from the diff body; application, compilation, and test execution remain for root review.

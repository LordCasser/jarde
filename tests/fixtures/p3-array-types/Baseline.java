import java.util.Arrays;

// The baseline driver of the array-types fixture: the committed original class is run in a
// controlled way, and this is what its own bytecode does — the answers the executed comparison's
// generated side must match. The comparison compiles it beside the sample and asserts these exact
// lines (the `## What the comparison answered` table in README.md records them).
//
// The input set is the one `tests/p3_execution_comparison.rs` calls the members with: `null` for
// every array parameter (`sample_values` has no value for an array type, so an array parameter's
// default is `null`) plus the one `byte[]` argument `echoed`'s own calls state. A trace cannot
// compare an `Object[]` — the helper would print a hash — so the arrays are printed element by
// element here, while the trace compares a `byte[]` result by its class.
public class Baseline {
    public static void main(String[] args) {
        System.out.println("copy(null)=" + ArrayTypes.copy(null));
        System.out.println(
                "echoed({1, 2})=" + Arrays.toString(ArrayTypes.echoed(new byte[] {1, 2})));
        System.out.println("named(null)=" + ArrayTypes.named(null));
        System.out.println("grid(null)=" + ArrayTypes.grid(null));
        System.out.println("table(null)=" + ArrayTypes.table(null));
        System.out.println("text(\"r\")=" + ArrayTypes.text("r"));
        System.out.println("text(null)=" + ArrayTypes.text(null));
    }
}

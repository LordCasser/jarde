// The baseline driver of the receiver-grouping fixture: the committed original class is run in a
// controlled way, and this is what its own bytecode does — the answers the executed comparison's
// generated side must match. The comparison compiles it beside the sample and asserts these exact
// lines (the `## What the comparison answered` table in README.md records them).
public class Baseline {
    public static void main(String[] args) {
        // The divergence case first: `("a", "bc")` is the input the review measured — the class
        // answers "bc" and the text that dropped the receiver's group answered "ac".
        System.out.println("call(\"a\", \"bc\")=" + ReceiverGrouping.call("a", "bc"));
        // The same body on the inputs the default comparison set uses for a `String`: `("r", "r")`
        // is where the two texts happen to agree, which is why the pair above is the finding.
        System.out.println("call(\"r\", \"r\")=" + ReceiverGrouping.call("r", "r"));
        System.out.println("call(null, \"bc\")=" + ReceiverGrouping.call(null, "bc"));
        System.out.println("length(\"a\", \"bc\")=" + ReceiverGrouping.length("a", "bc"));
        System.out.println("nested(\"a\", \"bc\", \"def\")=" + ReceiverGrouping.nested("a", "bc", "def"));
        System.out.println("plain(\"  a \")=[" + ReceiverGrouping.plain("  a ") + "]");
        System.out.println("chained(\"  a \")=" + ReceiverGrouping.chained("  a "));
        System.out.println("same(\"a\", \"b\", \"c\")=" + ReceiverGrouping.same("a", "b", "c"));
        System.out.println("argument(\"a\", \"bc\")=" + ReceiverGrouping.argument("a", "bc"));
    }
}

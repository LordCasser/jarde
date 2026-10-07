// `recover-statement-position-news`'s negatives: the statement-position constructions whose
// arguments carry an effect of their own, and the registered `chained` boundary. Every one of them
// keeps the refusal it had before this change, verbatim.
public class SPN {
    int n;

    SPN() { System.out.println("spn0"); }

    SPN(int v) { n = v; }

    static int probe() { System.out.println("probe"); return 1; }

    static SPN holder = new SPN(5);

    // A real invocation at an argument position: the frozen CST counterexample's family, kept out
    // of the statement position by this change's own criterion.
    static void callArgument() { new SPN(probe()); }

    // A field read at an argument position: `getfield`/`getstatic` trigger the declaring class's
    // `<clinit>`, which is an effect the statement position does not order.
    static void fieldArgument() { new SPN(holder.n); }

    // The strict enumeration's boundary: an arithmetic argument is neither a constant, a direct
    // read nor a proved construction, so the statement keeps its refusal.
    static void arithmeticArgument(int x) { new SPN(x + 1); }

    // The registered `chained` boundary: the argument is `new SPN(1).n`, a field read over a
    // construction that completes inside this one.
    static void chained() { new SPN(new SPN(1).n); }

    public static void main(String[] args) {
        callArgument();
        fieldArgument();
        arithmeticArgument(2);
        chained();
        System.out.println(holder.n);
    }
}

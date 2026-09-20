// The baseline driver of the nested-arithmetic fixture: the committed original class is run in a
// controlled way, and this is what its own bytecode does — the answers the executed comparison's
// generated side must match. The comparison compiles it beside the sample and asserts these exact
// lines (the `## What the comparison answered` table in README.md records them).
public class Baseline {
    public static void main(String[] args) {
        // The divergence case first: `inverse32(-1)` is the input the benchmark campaign measured —
        // the class answers -1 and the text that lost its grouping answered -81.
        System.out.println("inverse32(-1)=" + ModLike.inverse32(-1));
        System.out.println("inverse32(7)=" + ModLike.inverse32(7));
        System.out.println("inverse32(0)=" + ModLike.inverse32(0));
        // One line per shape, with inputs that exercise its own operator combination.
        System.out.println("scaledDifference(3, 5)=" + ModLike.scaledDifference(3, 5));
        System.out.println("nestedDifference(10, 4, 7)=" + ModLike.nestedDifference(10, 4, 7));
        System.out.println("nestedQuotient(20, 3, 4)=" + ModLike.nestedQuotient(20, 3, 4));
        System.out.println("differenceOfSum(10, 3)=" + ModLike.differenceOfSum(10, 3));
        System.out.println("productOfSum(2, 3, 4)=" + ModLike.productOfSum(2, 3, 4));
        System.out.println("sumOfProducts(2, 3, 4)=" + ModLike.sumOfProducts(2, 3, 4));
        System.out.println("leftNestedSum(2, 3, 4)=" + ModLike.leftNestedSum(2, 3, 4));
    }
}

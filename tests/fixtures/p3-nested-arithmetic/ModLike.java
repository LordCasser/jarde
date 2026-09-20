// The controlled sample of the nested-arithmetic defect: an arithmetic operand whose own value is
// another arithmetic's result, in the four shapes that lose their grouping when the expression tree
// is printed without parentheses, plus the two shapes that need none and must not gain any.
//
// `inverse32` is the shape the benchmark campaign reproduced in `org.bouncycastle.math.raw.Mod`
// (`d = -1` makes the original answer `-1` and the ungrouped text `-81`); the other methods are the
// small shapes the same defect covers, each with one operator combination.
public class ModLike {
    public static int inverse32(int d) {
        int i = d;
        i = i * (2 - d * i);
        i = i * (2 - d * i);
        i = i * (2 - d * i);
        i = i * (2 - d * i);
        return i;
    }

    // a * (2 - b * a): the multiplication of a subtraction, whose right operand is itself a product.
    public static int scaledDifference(int a, int b) {
        return a * (2 - b * a);
    }

    // a - (b - c): a subtraction on the right of a subtraction, the two of equal precedence.
    public static int nestedDifference(int a, int b, int c) {
        return a - (b - c);
    }

    // a / (b * c): a product on the right of a division, the two of equal precedence.
    public static int nestedQuotient(int a, int b, int c) {
        return a / (b * c);
    }

    // a - (b + 1) * 2: the product of a sum, on the right of a subtraction.
    public static int differenceOfSum(int a, int b) {
        return a - (b + 1) * 2;
    }

    // (a + b) * c: a sum on the left of a multiplication, which binds tighter than the sum.
    public static int productOfSum(int a, int b, int c) {
        return (a + b) * c;
    }

    // a + b * c: a product on the right of a sum. The default precedence already states this tree,
    // so the text must stay unparenthesised.
    public static int sumOfProducts(int a, int b, int c) {
        return a + b * c;
    }

    // (a + b) + c: a sum on the left of a sum. Left associativity already states this tree, so the
    // text must stay unparenthesised.
    public static int leftNestedSum(int a, int b, int c) {
        return (a + b) + c;
    }
}

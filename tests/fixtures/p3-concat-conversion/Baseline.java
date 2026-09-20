/**
 * The original class's own answers for the input set the execution comparison uses.
 *
 * <p>Compiled by `tests/p3_execution_comparison.rs` in a temporary directory with the sample on its
 * classpath, and run as its own program: the lines it prints are the values the generated text has
 * to answer too, because the value evidence of this fixture is what the *class* does and not what
 * the recovered text looks like. The `(1, 2)` pair and the counter around the two observable members
 * are the finding's own inputs: `twoIntsThenString(1, 2)` is `"12!"` and not `"3!"`, and the counter
 * shows that `marked`'s two parts are evaluated once each, in the order the bytecode called them.
 */
public class Baseline {

    /** One observation per line, exactly as the comparison asserts them. */
    public static void main(String[] args) {
        System.out.println("twoIntsThenString(1, 2)=" + ConcatConversion.twoIntsThenString(1, 2));
        System.out.println("twoIntsThenString(7, 0)=" + ConcatConversion.twoIntsThenString(7, 0));
        System.out.println("twoIntsThenString(0, -1)=" + ConcatConversion.twoIntsThenString(0, -1));
        System.out.println("onePartIsASum(1, 2)=" + ConcatConversion.onePartIsASum(1, 2));
        System.out.println("onePartIsASum(7, 0)=" + ConcatConversion.onePartIsASum(7, 0));
        System.out.println("numericLast(\"a\", 1)=" + ConcatConversion.numericLast("a", 1));
        System.out.println("numericLast(\"\", 0)=" + ConcatConversion.numericLast("", 0));
        System.out.println("numericLast(null, 2)=" + ConcatConversion.numericLast(null, 2));
        System.out.println("allStrings(\"a\", \"b\")=" + ConcatConversion.allStrings("a", "b"));
        System.out.println("allStrings(\"\", \"x\")=" + ConcatConversion.allStrings("", "x"));
        System.out.println("allStrings(null, \"x\")=" + ConcatConversion.allStrings(null, "x"));
        System.out.println("booleanLiteral()=" + ConcatConversion.booleanLiteral());
        System.out.println("booleanParameter(true)=" + ConcatConversion.booleanParameter(true));
        System.out.println("booleanParameter(false)=" + ConcatConversion.booleanParameter(false));
        System.out.println("nullPart()=" + ConcatConversion.nullPart());
        System.out.println("objectPart(\"o\")=" + ConcatConversion.objectPart("o"));
        System.out.println("objectPart(null)=" + ConcatConversion.objectPart(null));

        int before = ConcatConversion.calls;
        String marked = ConcatConversion.marked();
        System.out.println(
                "marked()=" + marked + " calls=" + before + "->" + ConcatConversion.calls);

        before = ConcatConversion.calls;
        String failing = ConcatConversion.failing(2);
        System.out.println(
                "failing(2)=" + failing + " calls=" + before + "->" + ConcatConversion.calls);

        before = ConcatConversion.calls;
        try {
            String zero = ConcatConversion.failing(0);
            System.out.println("failing(0)=" + zero + " calls=" + before + "->"
                    + ConcatConversion.calls);
        } catch (RuntimeException thrown) {
            System.out.println("failing(0)=" + thrown.getClass().getName() + ": "
                    + thrown.getMessage() + " calls=" + before + "->" + ConcatConversion.calls);
        }

        System.out.println("markA()=" + ConcatConversion.markA() + " markB()="
                + ConcatConversion.markB() + " calls=" + ConcatConversion.calls);
        System.out.println("div(7, 7)=" + ConcatConversion.div(7, 7) + " div(-1, -1)="
                + ConcatConversion.div(-1, -1));
    }
}

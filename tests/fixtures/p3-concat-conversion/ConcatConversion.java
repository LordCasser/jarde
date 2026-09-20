/**
 * The P3 concatenation-conversion regression, compiled with a real compiler (`concat@1`, T4).
 *
 * <p>A verified `StringBuilder` chain is written as one `+` expression, and `+` and `append` convert
 * their operands the same way only while the text is in a **string context**. The pre-fix shape
 * built the expression by folding `+` from the first operand, so two adjacent numeric operands were
 * added as numbers:
 *
 * <pre>
 * source: new StringBuilder().append(a).append(b).append("!").toString()
 * text:   arg0 + arg1 + "!"            // (1, 2) answers "3!", the class answers "12!"
 * </pre>
 *
 * <p>Every member is one shape of that defect or a control the fix must not decorate:
 *
 * <ul>
 *   <li>{@link #twoIntsThenString(int, int)} — the reported shape: two parts that need conversion
 *       before the first `String` part;
 *   <li>{@link #onePartIsASum(int, int)} — **one** part whose value is itself an addition, which
 *       keeps its own group (`"" + (arg0 + arg1) + "!"`);
 *   <li>{@link #numericLast(String, int)} — the part needing conversion comes last, which already
 *       worked and must stay byte for byte the same text;
 *   <li>{@link #allStrings(String, String)} — no part needs conversion at all (the second control);
 *   <li>{@link #booleanLiteral()} — a `boolean` literal part: `append(true)` is `iconst_1`, and
 *       `"" + 1` is `"1"` where the class writes `"true"`;
 *   <li>{@link #booleanParameter(boolean)} — the same conversion for a value the descriptor proves
 *       `boolean`, which the value's own text already spells;
 *   <li>{@link #nullPart()} — `append((Object) null)`, whose conversion is `String.valueOf(Object)`;
 *   <li>{@link #objectPart(Object)} — a non-null object part, converted the same way;
 *   <li>{@link #marked()} — two parts whose evaluations are observable (a shared counter and two
 *       distinct values), so a reorder, a merge or a repeated evaluation changes the trace;
 *   <li>{@link #failing(int)} — a part that throws, so the order of the evaluations before it is
 *       observable in the counter the trace prints around the call;
 *   <li>{@link #markA()}/{@link #markB()}/{@link #div(int, int)} — the observable and throwing
 *       parts themselves.
 * </ul>
 */
public class ConcatConversion {

    /** How many times an observable part has been evaluated. */
    static int calls;

    /** Two numeric parts before a `String` part: `(1, 2)` answers `"12!"`. */
    public static String twoIntsThenString(int a, int b) {
        return new StringBuilder().append(a).append(b).append("!").toString();
    }

    /** One part that is itself an addition: `(1, 2)` answers `"3!"`, and the group must survive. */
    public static String onePartIsASum(int a, int b) {
        return new StringBuilder().append(a + b).append("!").toString();
    }

    /** The part needing conversion comes last: the text must stay `arg0 + arg1`. */
    public static String numericLast(String a, int b) {
        return new StringBuilder().append(a).append(b).toString();
    }

    /** Every part is already a `String`: the text must gain no empty string and no decoration. */
    public static String allStrings(String a, String b) {
        return new StringBuilder().append(a).append(b).toString();
    }

    /** A `boolean` literal part: `append(true)` writes `"true"`, `"" + 1` would write `"1"`. */
    public static String booleanLiteral() {
        return new StringBuilder().append(true).append("!").toString();
    }

    /** The same conversion for a value whose own evidence is `boolean`. */
    public static String booleanParameter(boolean flag) {
        return new StringBuilder().append(flag).append("!").toString();
    }

    /** A `null` part: `append((Object) null)` writes `"null"`. */
    public static String nullPart() {
        return new StringBuilder().append((Object) null).append("!").toString();
    }

    /** An object part, converted with `String.valueOf(Object)` — including a `null` value. */
    public static String objectPart(Object value) {
        return new StringBuilder().append(value).append("!").toString();
    }

    /** Two observable parts: each call moves the counter and returns its own value. */
    public static String marked() {
        return new StringBuilder().append(markA()).append(markB()).append("!").toString();
    }

    /** An observable part followed by a part that throws: the trace shows which ran. */
    public static String failing(int b) {
        return new StringBuilder().append(markA()).append(div(100, b)).append("!").toString();
    }

    /** The first observable part. */
    public static int markA() {
        calls = calls + 1;
        return 1;
    }

    /** The second observable part. */
    public static int markB() {
        calls = calls + 1;
        return 2;
    }

    /** The part that throws when `b` is zero. */
    public static int div(int a, int b) {
        return a / b;
    }
}

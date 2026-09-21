/**
 * The P3 required-conversion regression: a value whose **presented** type is not the type the
 * position it is written in requires (T5, C01/C02).
 *
 * <p>A `char`, a `byte` and a `short` share one slot shape with an `int`, so the frames cannot say
 * which of the four a position holds and the compiler inserts **no** instruction when it widens one
 * of them to `int`: `append((int) c)` and `int local = c;` are a `load` plus the consumer's own
 * instruction. A presentation that writes the value's own text there publishes the *other*
 * conversion — `"" + c` is a string concatenation of a **character** (`"A"`), where the bytecode
 * converted the code unit (`"65"`), and both texts compile:
 *
 * <pre>
 * source: new StringBuilder().append((int) c).append("!").toString()
 * text:   "" + arg0 + "!"        // c == 'A' answers "A!", the class answers "65!"
 * </pre>
 *
 * <p>Every member is one position that requires a type, one control that must not gain a
 * conversion, or the helper a position's requirement is stated by:
 *
 * <ul>
 *   <li>{@link #castPart(char)} — the reported shape: the `append`'s own parameter descriptor says
 *       `int` while the value it reads is a `char`;
 *   <li>{@link #castPartLast(String, char)} — the same part after a `String` part, so the chain is
 *       already a string concatenation and the empty string is not what decides the value;
 *   <li>{@link #intPart(int)} — the control: an `int` part needs no conversion, and its text must
 *       stay byte for byte the text it was;
 *   <li>{@link #argued(char)}, {@link #arguedByte(byte)} — a call argument: the callee's own
 *       descriptor takes `int` and the argument is a `char`/`byte`, which the JVM widens without an
 *       instruction (`widen`'s body is what makes the argument's value observable);
 *   <li>{@link #returned(char)}, {@link #returnedShort(short)} — a `return` in a member whose own
 *       descriptor returns `int` from a `char`/`short` value;
 *   <li>{@link #kept(char)} — the control: the member returns the type the value is presented as, so
 *       nothing converts;
 *   <li>{@link #declared(char)} — a declaration filled with a `char` value where the frame states
 *       `int` (the local's own declaration is what the value has to meet);
 *   <li>{@link #assigned(char)} — the same write after the declaration is already written;
 *   <li>{@link #written(char)} — a `putstatic` into a field whose own descriptor is `int`;
 *   <li>{@link #widen(int)} — the callee of the argument samples.
 * </ul>
 */
public class RequiredConversions {

    /** The field {@link #written(char)} stores into: its descriptor is `I`. */
    static int field;

    /** T5: `append((int) c)` — the class answers `"65!"` for `'A'`, `"" + c` answers `"A!"`. */
    public static String castPart(char c) {
        return new StringBuilder().append((int) c).append("!").toString();
    }

    /** The same converted part comes after a `String` part: the chain's `+` is already a string. */
    public static String castPartLast(String a, char c) {
        return new StringBuilder().append(a).append((int) c).toString();
    }

    /** The control: an `int` part meets the `int` parameter, so no conversion may be written. */
    public static String intPart(int x) {
        return new StringBuilder().append(x).append("!").toString();
    }

    /** The callee of the argument samples: its own descriptor states the parameter is an `int`. */
    public static int widen(int x) {
        return x;
    }

    /** A `char` argument to an `int` parameter, which the JVM widens with no instruction. */
    public static int argued(char c) {
        return widen(c);
    }

    /** A `byte` argument to the same `int` parameter. */
    public static int arguedByte(byte b) {
        return widen(b);
    }

    /** A member whose own descriptor returns `int`, returning a `char` value. */
    public static int returned(char c) {
        return c;
    }

    /** The same value from a `short`, which shares the same slot shape. */
    public static int returnedShort(short s) {
        return s;
    }

    /** The control: the member returns the type the value is presented as. */
    public static char kept(char c) {
        return c;
    }

    /** A declaration filled with a `char`: the local's own declared type is what the value meets. */
    public static int declared(char c) {
        int local = c;
        return local;
    }

    /** The same value written after the declaration is already written elsewhere. */
    public static int assigned(char c) {
        int local = 0;
        local = c;
        return local;
    }

    /** A field whose own descriptor is `int`, written with a `char` value. */
    public static int written(char c) {
        field = c;
        return field;
    }
}

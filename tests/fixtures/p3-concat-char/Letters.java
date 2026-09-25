/**
 * The P3 task 2c.19 sample: `append(C)` is a character operand of a string `+` (`concat@1`).
 *
 * <p>A verified chain is written as one `+` expression, and the emitter starts that expression in a
 * string context (`"" +`) whenever its first part is not already a `String`. So the code unit an
 * `append(C)` instruction pushes is concatenated — `"x" + c` writes the character, it is never
 * added as a number — and the overload is one `+` reproduces. `char[]` and `CharSequence` are not:
 * `+` writes an array's `toString` and converts a sequence with `String.valueOf`, so a chain
 * carrying either is still refused whole.
 *
 * <ul>
 *   <li>{@link #letter(char)} — `append(C)` after a `String` part: the text keeps the string context
 *       the first part already gives it (`"x" + arg0`);
 *   <li>{@link #only(char)} — `"" + c`, which `javac --release 8` lowers to `append("")` and then
 *       `append(C)`: the empty string is a part of its own, and the text is `"" + arg0` — never the
 *       bare `arg0` a lost conversion would write.
 * </ul>
 */
public class Letters {

    /** One character after a `String` part: the text keeps `"x" + arg0`. */
    static String letter(char c) {
        return "x" + c;
    }

    /** One character from an empty-string start: the text keeps `"" + arg0`. */
    static String only(char c) {
        return "" + c;
    }
}

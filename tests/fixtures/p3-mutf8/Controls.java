/**
 * The P3 task 6.4 sample: `controls` returns a string constant holding one NUL.
 *
 * <p>The constant pool holds that string as the bytes `61 C0 80 62`: Modified UTF-8 writes U+0000
 * in the two-byte form `C0 80` rather than as a raw zero byte. A reader that decodes the `Utf8`
 * entry as standard UTF-8 refuses that overlong form and makes two replacement characters of it, so
 * the NUL the class file states is lost. See `README.md` for the compiler, the command, the bytes
 * and the digests.
 */
public class Controls {

    /** U+0000 inside a string that is otherwise ASCII. */
    static String controls() {
        return "a\u0000b";
    }
}

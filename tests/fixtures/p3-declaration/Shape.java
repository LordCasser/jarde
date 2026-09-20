/**
 * The interface half of the P3 declaration fixture: the member forms a class file's own flags
 * decide, in the one class kind that a member's flags cannot stand in for.
 *
 * <p>A `default` method is not a flag of its own: JVMS 4.6 gives an interface's methods `public`,
 * `static`, `abstract` and the rest, and "the `default` keyword" is exactly "an interface's method
 * that is neither `static` nor `abstract`". Distinguishing {@link #scaled(int)} from a class's
 * ordinary method therefore takes `ACC_INTERFACE` on the **declaring class** — which is why the
 * three members below are one file: the same `public` flag means a `default` method here and an
 * instance method in {@link Holder}.
 *
 * <p>Compiled by javac 23.0.1 with `--release 8 -g:none`; see this directory's README for the
 * command, the digest and the bytecode of every member.
 */
public interface Shape {

    /** An interface's `abstract` method: the class file declares no `Code` for it at all. */
    int sides();

    /** An interface's `default` method: `public`, neither `static` nor `abstract`. */
    default int scaled(int factor) {
        return sides() * factor;
    }

    /** An interface's `static` method (Java 8 and later). */
    static int sum(int a, int b) {
        return a + b;
    }
}

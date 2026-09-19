/**
 * The four shapes of the P3 1.3d regression, compiled with a real compiler.
 *
 * <p>Two of them are the same *statement* in the generated text — a write to slot 0 followed by a
 * return of slot 0 — and they differ only in what the bytecode does:
 *
 * <ul>
 *   <li>{@link #post(int)} loads the slot, increments it and returns the value it *loaded*, so the
 *       slot's name at the return denotes the incremented value and must not be written there;
 *   <li>{@link #bump(int)} and {@link #doubleIt(int)} write the slot and then load it again, so the
 *       slot's name at the return denotes exactly the value being returned and must be written.
 * </ul>
 *
 * <p>{@link #cast()} is the P3-R2 shape: a value a verified consumer presents while the invocation
 * that produced it must survive in the answer. {@link #make()} is that producer, and the static
 * field counts its invocations so a test can tell one call from two.
 */
public final class LocalRewrite {

    static int calls;

    static Object make() {
        calls++;
        return "ok";
    }

    /** Returns the value the slot held before the increment: {@code post(7) == 7}. */
    public static int post(int x) {
        return x++;
    }

    /** Writes the slot, then reads it again: {@code bump(7) == 8}. */
    public static int bump(int x) {
        x = x + 1;
        return x;
    }

    /** The same write-then-read shape for an operation {@code iinc} cannot spell. */
    public static int doubleIt(int x) {
        x = x * 2;
        return x;
    }

    /** The post-increment value assigned to another local: a **store** consumes the old value. */
    public static int saved(int x) {
        int y = x++;
        return y;
    }

    /** A **branch** consumes the old value: the test runs after the increment wrote the slot. */
    public static int conditional(int x) {
        if (x++ > 0) {
            return 1;
        }
        return 0;
    }

    /** A value loaded before a loop whose body rewrites the slot, returned after it. */
    public static int loopAcross(int x, int n) {
        int y = x;
        while (n > 0) {
            x = x + 1;
            n = n - 1;
        }
        return y;
    }

    /** A cast with a producer behind it, which no rule of this slice presents. */
    public static String cast() {
        return (String) make();
    }
}

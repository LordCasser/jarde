/**
 * The class half of the P3 declaration fixture: the member forms a class file decides by its own
 * names and flags, and the class-level fact a constructor's own prologue needs.
 *
 * <p>{@link #Holder(int)} is the `<init>` of JVMS 2.9, and it writes the field it is handed on its
 * **own uninitialized `this`** before it has called any constructor — a shape JVMS 4.10.1.9 lets
 * through only a `Fieldref` that names the class being constructed, so reading it at all takes the
 * declaring class's own name. {@link #TOKEN} is initialised by a call rather than by a constant, so
 * the class file really declares a `<clinit>`; {@link #value()} and {@link #of(int)} are the
 * ordinary instance and static members the same class declares.
 *
 * <p>Compiled by javac 23.0.1 with `--release 8 -g:none`; see this directory's README for the
 * command, the digests and the bytecode of every member.
 */
public final class Holder {

    /** A non-constant static initializer: the class file's own `<clinit>`. */
    static final Object TOKEN = new Object();

    private final int value;

    /** A constructor that writes the field on its uninitialized `this`. */
    public Holder(int value) {
        this.value = value;
    }

    /** An ordinary instance method of a class. */
    public int value() {
        return value;
    }

    /** An ordinary static method of a class, and a construction site of its own class. */
    public static Holder of(int value) {
        return new Holder(value);
    }
}

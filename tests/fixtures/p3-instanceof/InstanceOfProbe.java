/**
 * Java 8 source for the permanent instanceof recovery fixture.
 *
 * Every method is a positive, whole-class case.  The explicit Object widening in
 * widenedString and called is source-level context that javac erases from the bytecode; a recovery
 * must put that safe widening back instead of narrowing the operand to the tested type.
 */
public class InstanceOfProbe {
    public static boolean objectString(Object value) {
        return value instanceof String;
    }

    public static boolean nullValue() {
        return (Object) null instanceof String;
    }

    public static boolean objectRunnable(Object value) {
        return value instanceof Runnable;
    }

    public static boolean primitiveArray(Object value) {
        return value instanceof int[];
    }

    public static boolean referenceArray(Object value) {
        return value instanceof String[];
    }

    public static boolean multiArray(Object value) {
        return value instanceof String[][];
    }

    public static boolean widenedString(String value) {
        return (Object) value instanceof Integer;
    }

    public static boolean called() {
        return (Object) InstanceOfSupport.value() instanceof Integer;
    }

    public static boolean local(Object value) {
        boolean found = value instanceof String;
        return found;
    }

    public static boolean parameter(Object value) {
        return keep(value instanceof String);
    }

    public static int branch(Object value) {
        if (value instanceof String) {
            return 1;
        }
        return 0;
    }

    public static boolean functional() {
        return ((Runnable) InstanceOfProbe::empty) instanceof Runnable;
    }

    private static boolean keep(boolean value) {
        return value;
    }

    private static void empty() {}
}

public final class InstanceOfMerge {
    static int calls;
    static Object value(Object input) { calls++; return input; }
    static boolean inverted(Object input) {
        return value(input) instanceof String ? false : true;
    }
    static boolean direct(Object input) {
        return value(input) instanceof String;
    }
}

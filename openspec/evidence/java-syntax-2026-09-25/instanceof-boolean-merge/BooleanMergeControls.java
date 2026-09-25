public final class BooleanMergeControls {
    static int calls;
    static Object value(Object x) { calls++; return x; }
    static boolean positive(Object x) { return value(x) instanceof String ? true : false; }
    static boolean negative(Object x) { return value(x) instanceof String ? false : true; }
}

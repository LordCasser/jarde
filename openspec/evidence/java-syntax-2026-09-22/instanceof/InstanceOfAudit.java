public class InstanceOfAudit {
    public static int calls;
    public static Object source(Object value) { calls = calls + 1; return value; }
    public static boolean string(Object value) { return value instanceof String; }
    public static boolean number(Object value) { return value instanceof Number; }
    public static boolean runnable(Object value) { return value instanceof Runnable; }
    public static boolean primitiveArray(Object value) { return value instanceof int[]; }
    public static boolean referenceArray(Object value) { return value instanceof String[]; }
    public static boolean multiArray(Object value) { return value instanceof String[][]; }
    public static boolean called(Object value) { return source(value) instanceof String; }
    public static boolean local(Object value) { boolean found = value instanceof String; return found; }
    public static int branch(Object value) { if (value instanceof String) return ((String) value).length(); return -1; }
    public static boolean negated(Object value) { return !(value instanceof String); }
}

public class ArrayReferenceOverloadTarget {
    public static String overload(Object[] value) {
        return "Object[]";
    }

    public static String overload(String[] value) {
        return "String[]";
    }

    public static String explicitObjectTarget(String[] value) {
        return overload((Object[]) value);
    }

    public static String localObjectTarget(String[] value) {
        Object[] widened = value;
        return overload(widened);
    }

    public static String implicitStringTarget(String[] value) {
        return overload(value);
    }
}

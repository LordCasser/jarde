public class ArrayReferenceOverloadTarget {
    private static int evaluations;

    public static String overload(Object[] value) {
        return "Object[]";
    }

    public static String overload(String[] value) {
        return "String[]";
    }

    public static String localObjectTarget(String[] value) {
        Object[] widened = value;
        return overload(widened);
    }

    public static String implicitStringTarget(String[] value) {
        return overload(value);
    }

    private static String[] effectfulArray() {
        evaluations++;
        return new String[0];
    }

    public static String effectfulObjectTarget() {
        Object[] widened = effectfulArray();
        return overload(widened);
    }

    public static int effectfulEvaluationCount() {
        return evaluations;
    }
}

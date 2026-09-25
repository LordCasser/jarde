public class ArrayReferenceConversions {
    public static String overload(Object value) {
        return "Object";
    }

    public static String overload(Object[] value) {
        return "Object[]";
    }

    public static String overload(Object[][] value) {
        return "Object[][]";
    }

    public static String typedIntMatrix(int[][] value) {
        return overload(value);
    }

    public static String typedStringArray(String[] value) {
        return overload(value);
    }

    public static String typedStringMatrix(String[][] value) {
        return overload(value);
    }

    public static String widenedObject(Object value) {
        return overload(value);
    }

    public static String widenedObjectArray(Object[] value) {
        return overload(value);
    }

    public static String exactIntMatrix(int[][] value) {
        return "int[][]:" + value.length;
    }

    public static String exactStringArray(String[] value) {
        return "String[]:" + value.length;
    }

    public static String exactStringMatrix(String[][] value) {
        return "String[][]:" + value.length;
    }

    public static String exactObjectArray(Object[] value) {
        return "Object[]:" + value.length;
    }

    public static String exactObjectMatrix(Object[][] value) {
        return "Object[][]:" + value.length;
    }

    public static String describeObjectArray(Object[] value) {
        return value.getClass().getName() + ":" + value.length;
    }

    public static String describeObjectMatrix(Object[][] value) {
        return value.getClass().getName() + ":" + value.length;
    }

    public static String writeObjectArray(Object[] value) {
        value[0] = new Object();
        return "stored";
    }

    public static String writeObjectMatrix(Object[][] value) {
        value[0] = new Object[0];
        return "stored";
    }
}

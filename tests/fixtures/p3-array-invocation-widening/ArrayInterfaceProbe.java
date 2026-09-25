import java.io.Serializable;

public class ArrayInterfaceProbe {
    static String cloneArray(Cloneable[] value) {
        return "Cloneable[]:" + value.getClass().getName();
    }

    static String serialArray(Serializable[] value) {
        return "Serializable[]:" + value.getClass().getName();
    }

    static String cloneObject(Cloneable value) {
        return "Cloneable:" + value.getClass().getName();
    }

    public static String intMatrix(int[][] value) {
        return cloneArray(value);
    }

    public static String stringMatrix(String[][] value) {
        return serialArray(value);
    }

    public static String intVector(int[] value) {
        return cloneObject(value);
    }
}

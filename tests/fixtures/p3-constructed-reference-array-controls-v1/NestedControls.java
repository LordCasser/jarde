public class NestedControls {
    static String mark(String value) {
        System.out.print("mark:");
        System.out.println(value);
        return value;
    }

    static Object[] nested() {
        return new Object[]{new StringBuilder(new StringBuilder(mark("nested")))};
    }

    public static void main(String[] args) {
        Object[] values = nested();
        Object value = values[0];
        System.out.println(value);
    }
}

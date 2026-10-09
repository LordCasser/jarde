public class BoundaryControls {
    static String mark(String value) {
        System.out.print("mark:");
        System.out.println(value);
        return value;
    }

    static int number(String value) {
        mark(value);
        return 1;
    }

    static Object[] firstThenUnsupportedStructure() {
        return new Object[]{
            new StringBuilder(mark("first")),
            new Long((long) number("second"))
        };
    }

    static void old(Object[] values) {
        values[0] = new StringBuilder(mark("old"));
    }

    static Object[] repeated() {
        Object[] values = new Object[2];
        values[0] = new StringBuilder(mark("repeated-first"));
        values[0] = new StringBuilder(mark("repeated-second"));
        return values;
    }

    static Object[] descending() {
        Object[] values = new Object[2];
        values[1] = new StringBuilder(mark("descending-second"));
        values[0] = new StringBuilder(mark("descending-first"));
        return values;
    }

    static Object[] crossBlock(boolean store) {
        Object[] values = new Object[1];
        StringBuilder element = new StringBuilder(mark("cross-block"));
        if (store) {
            values[0] = element;
        }
        return values;
    }

    static Number[] closedNumberBoundary() {
        return new Number[]{
            new Integer(mark("1")),
            new java.math.BigDecimal(mark("2"))
        };
    }
}

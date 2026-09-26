public class VarargsCalls {
    static int effects;
    static String order;

    static int mark(int value) {
        effects++;
        order += value;
        if (value == 9) {
            throw new IllegalStateException("stop");
        }
        return value;
    }

    static int count(int... values) {
        return values.length;
    }

    static int objects(Object... values) {
        return values.length;
    }

    static String shape(Object... values) {
        return values.getClass().getName() + "/" + values.length;
    }

    static int strings(String... values) {
        return values.length;
    }

    static int plain(int[] values) {
        return values.length;
    }

    static int overloaded(Object... values) {
        return values.length;
    }

    static int overloaded(String value) {
        return value.length();
    }

    static int ordered() {
        return count(mark(1), mark(2), mark(3));
    }

    static int oneString() {
        return strings("one");
    }

    static int objectValues() {
        return objects("a", "b");
    }

    static int explicitVarargsArray() {
        return count(new int[] { mark(4), mark(5) });
    }

    static int ordinaryArray() {
        return plain(new int[] { mark(6), mark(7) });
    }

    static int heldArray() {
        int[] values = new int[] { mark(8), mark(1) };
        return count(values);
    }

    static int overloadedArray() {
        return overloaded(new Object[] { "x" });
    }

    static String nullElement() {
        return shape((Object) null);
    }

    static String arrayElement() {
        return shape(new String[] { "x" });
    }

    static int exceptionOrder() {
        return count(mark(1), mark(9), mark(2));
    }
}

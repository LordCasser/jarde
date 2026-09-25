
/* JADX INFO: loaded from: NestedArrayInitializer.class */
public final class NestedArrayInitializer {
    static int calls;
    static int trace;

    static int element(int i) {
        calls++;
        trace = (trace * 10) + i;
        return i;
    }

    static int[][] dynamic() {
        return new int[][]{new int[]{element(1), element(2)}, new int[]{element(3)}};
    }

    static String[][] literal() {
        return new String[][]{new String[]{"a", "b"}, new String[]{"c"}};
    }
}

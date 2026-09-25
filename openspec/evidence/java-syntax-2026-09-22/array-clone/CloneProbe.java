public final class CloneProbe {
    public static int[] ints(int[] source) {
        return source.clone();
    }

    public static String[] refs(String[] source) {
        return source.clone();
    }

    public static int[][] outer(int[][] source) {
        return source.clone();
    }

    public static Object[] widen(String[] source) {
        return source.clone();
    }

    public static int accept(Object value) {
        return 1;
    }

    public static int accept(int[] value) {
        return 2;
    }

    public static int overload(int[] source) {
        return accept(source.clone());
    }
}

package reference;

public final class LoadNegatives {
    private static int effects;
    public static Object multi(Object input) {
        Object local;
        return local = input;
    }

    public static void effect() { effects = 1; }

    public static Object crossEffect(Object input) {
        Object local;
        effect();
        return local = input;
    }
}

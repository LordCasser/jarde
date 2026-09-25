package nested;

/* JADX INFO: loaded from: missing-target.jar:nested/SimpleOuter.class */
public final class SimpleOuter {
    public static String trace = "";
    private final int bias;

    public SimpleOuter(int i) {
        this.bias = i;
    }

    public static int mark(String str, int i) {
        trace += str;
        return i;
    }
}

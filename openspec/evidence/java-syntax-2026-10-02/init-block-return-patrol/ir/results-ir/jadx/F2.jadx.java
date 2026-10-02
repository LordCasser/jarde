package defpackage;

/* JADX INFO: loaded from: F2.class */
public class F2 {
    static int value;
    static int other;

    static int init() {
        try {
            return Integer.parseInt("42");
        } catch (NumberFormatException e) {
            throw new RuntimeException("wrap", e);
        }
    }

    public static void main(String[] strArr) {
        System.out.println(value + ":" + other);
    }

    static {
        if (Boolean.getBoolean("boom")) {
            throw new IllegalStateException("clinit-fail");
        }
        value = 5;
        other = init();
    }
}

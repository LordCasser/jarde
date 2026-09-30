package defpackage;

/* JADX INFO: loaded from: C2.class */
public class C2 {
    public static String alias(int i) {
        if (i != 0) {
            return "fine";
        }
        try {
            throw new IllegalStateException("zero");
        } catch (IllegalStateException e) {
            throw new RuntimeException("w:" + e.getMessage(), e);
        }
    }

    public static void main(String[] strArr) {
        try {
            alias(0);
        } catch (RuntimeException e) {
            System.out.println(e.getMessage() + "/" + (e.getCause() instanceof IllegalStateException));
        }
        System.out.println(alias(1));
    }
}

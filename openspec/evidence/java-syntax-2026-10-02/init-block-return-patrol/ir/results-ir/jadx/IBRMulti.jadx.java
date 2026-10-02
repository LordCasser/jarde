package defpackage;

/* JADX INFO: loaded from: IBRMulti.class */
public class IBRMulti {
    static int a = 1;
    static int b;
    static int c;

    public static void main(String[] strArr) {
        System.out.println(a + ":" + b + ":" + c);
    }

    static {
        b = 2;
        if (Boolean.getBoolean("ibr-multi-boom")) {
            throw new IllegalStateException("multi-fail");
        }
        a += b;
        c = a * 3;
        b = a + c;
    }
}

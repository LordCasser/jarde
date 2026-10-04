package defpackage;

/* JADX INFO: loaded from: LC.class */
public class LC {
    static int plain(int i) {
        return new LC.1L(i).run();
    }

    static int two(int i, int i2) {
        return new LC.2L(i, i2).run();
    }

    static int local(int i) {
        return new LC.3L(i * 2).run();
    }

    static int named(int i) {
        return new LC.1Named(i).f();
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(plain(3) + "/" + two(1, 2) + "/" + local(4) + "/" + named(5));
    }
}

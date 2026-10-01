package defpackage;

/* JADX INFO: loaded from: B4.class */
public class B4 {
    public static String v1(int i) {
        return (((((i | 16) & 60) ^ 8) << 1) >> 2) + ":" + (((i & 1) == 0 || (i & 2) == 0) ? false : true);
    }

    public static String v2(int i) {
        return (i | 16) + ":" + (((i & 1) == 0 || (i & 2) == 0) ? false : true) + ":" + (((i & 4) == 0 && (i & 8) == 0) ? false : true);
    }

    public static String v3(int i) {
        return (((i & 1) == 0 || (i & 2) == 0) ? false : true) + ":" + (((i & 4) == 0 && (i & 8) == 0) ? false : true);
    }

    public static void main(String[] strArr) {
        System.out.println(v1(27));
        System.out.println(v2(27));
        System.out.println(v3(27));
    }
}

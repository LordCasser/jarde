package defpackage;

/* JADX INFO: loaded from: B2.class */
public class B2 {
    static long seed = 78187493530L;

    public static long longOps(long j) {
        return (((j ^ 65535) | ((j & 255) << 8)) & (-16)) >>> 2;
    }

    public static String compound(int i) {
        return (((((i | 16) & 60) ^ 8) << 1) >> 2) + ":" + (((i & 1) == 0 || (i & 2) == 0) ? false : true) + ":" + (((i & 4) == 0 && (i & 8) == 0) ? false : true);
    }

    public static void main(String[] strArr) {
        System.out.println(longOps(seed));
        System.out.println(compound(27));
    }
}

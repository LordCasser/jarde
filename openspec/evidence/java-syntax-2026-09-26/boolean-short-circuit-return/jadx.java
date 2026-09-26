package defpackage;

/* JADX INFO: loaded from: BoolValue.class */
public final class BoolValue {
    static int calls;

    static boolean positive(int i) {
        calls++;
        return i > 0;
    }

    public static boolean and(int i, int i2) {
        return i > 0 && i2 > 0;
    }

    public static boolean or(int i, int i2) {
        return i > 0 || i2 > 0;
    }

    public static boolean effectfulAnd(int i, int i2) {
        return positive(i) && positive(i2);
    }

    public static boolean effectfulOr(int i, int i2) {
        return positive(i) || positive(i2);
    }

    public static void main(String[] strArr) {
        for (int i = -1; i <= 1; i += 2) {
            for (int i2 = -1; i2 <= 1; i2 += 2) {
                calls = 0;
                System.out.println(i + "," + i2 + ":" + and(i, i2) + "," + or(i, i2) + "," + effectfulAnd(i, i2) + "," + effectfulOr(i, i2) + "," + calls);
            }
        }
    }
}

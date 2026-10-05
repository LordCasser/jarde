package defpackage;

/* JADX INFO: loaded from: BW.class */
public class BW {
    static boolean xor(boolean z, boolean z2) {
        return z ^ z2;
    }

    static int ixor(int i, int i2) {
        return i ^ i2;
    }

    static boolean mix(boolean[] zArr) {
        boolean z = false;
        for (boolean z2 : zArr) {
            z ^= z2;
        }
        return z;
    }

    static int shConst(int i) {
        return i << 33;
    }

    static int shFold() {
        return Integer.MIN_VALUE;
    }

    static boolean andNot(boolean z, boolean z2) {
        return z & (!z2);
    }

    static int bits(int i) {
        int i2 = 0;
        while (i != 0) {
            i &= i - 1;
            i2++;
        }
        return i2;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + xor(true, false) + "/" + ixor(6, 3) + "/" + mix(new boolean[]{true, true, true}) + "/" + shConst(1) + "/" + shFold() + "/" + andNot(true, true) + "/" + bits(11));
    }
}

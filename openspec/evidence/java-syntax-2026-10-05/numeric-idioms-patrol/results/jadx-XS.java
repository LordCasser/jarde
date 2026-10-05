package defpackage;

/* JADX INFO: loaded from: XS.class */
public class XS {
    static int[] xorSwap(int[] iArr) {
        iArr[0] = iArr[0] ^ iArr[1];
        iArr[1] = iArr[1] ^ iArr[0];
        iArr[0] = iArr[0] ^ iArr[1];
        return iArr;
    }

    static int min3(int i, int i2, int i3) {
        if (i < i2) {
            return i < i3 ? i : i3;
        }
        return i2 < i3 ? i2 : i3;
    }

    static int max3(int i, int i2, int i3) {
        if (i > i2) {
            return i > i3 ? i : i3;
        }
        return i2 > i3 ? i2 : i3;
    }

    static int clamp(int i, int i2, int i3) {
        if (i < i2) {
            return i2;
        }
        return i > i3 ? i3 : i;
    }

    static int abs(int i) {
        return i < 0 ? -i : i;
    }

    static int gcd(int i, int i2) {
        while (i2 != 0) {
            int i3 = i % i2;
            i = i2;
            i2 = i3;
        }
        return i;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + xorSwap(new int[]{3, 8})[0] + "/" + min3(4, 2, 9) + "/" + max3(4, 2, 9) + "/" + clamp(15, 0, 10) + "/" + abs(-7) + "/" + gcd(48, 18));
    }
}

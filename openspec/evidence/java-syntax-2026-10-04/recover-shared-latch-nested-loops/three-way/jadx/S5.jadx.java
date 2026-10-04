package defpackage;

/* JADX INFO: loaded from: S5.class */
public class S5 {
    public static int outerContinueInner(int i) {
        int i2 = 0;
        for (int i3 = 0; i3 < i; i3++) {
            if (i3 % 2 != 0) {
                for (int i4 = 0; i4 < i3; i4++) {
                    i2 += i4;
                }
            }
        }
        return i2;
    }

    public static int outerContinueOnly(int i) {
        int i2 = 0;
        for (int i3 = 0; i3 < i; i3++) {
            if (i3 % 2 != 0) {
                i2 += i3;
            }
        }
        return i2;
    }

    public static int innerOnly(int i) {
        int i2 = 0;
        for (int i3 = 0; i3 < i; i3++) {
            for (int i4 = 0; i4 < i3; i4++) {
                i2 += i4;
            }
        }
        return i2;
    }

    public static int labeledOuter(int i) {
        int i2 = 0;
        for (int i3 = 0; i3 < i; i3++) {
            if (i3 % 2 != 0) {
                for (int i4 = 0; i4 < i3; i4++) {
                    i2 += i4;
                }
            }
        }
        return i2;
    }

    public static void main(String[] strArr) {
        System.out.println(outerContinueInner(6));
        System.out.println(outerContinueOnly(6));
        System.out.println(innerOnly(6));
        System.out.println(labeledOuter(6));
    }
}

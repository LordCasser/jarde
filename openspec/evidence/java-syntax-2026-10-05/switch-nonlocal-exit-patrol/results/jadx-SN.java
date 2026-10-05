package defpackage;

/* JADX INFO: loaded from: SN.class */
public class SN {
    static int retInSwitchNoTail(int[] iArr) {
        int i = 0;
        for (int i2 : iArr) {
            switch (i2) {
                case 3:
                    return i + 100;
                default:
                    i += i2;
                    break;
            }
        }
        return i;
    }

    static int retInSwitchTail(int[] iArr) {
        int i = 0;
        for (int i2 : iArr) {
            switch (i2) {
                case 3:
                    return i + 100;
                default:
                    i = i + i2 + 10;
                    break;
            }
        }
        return i;
    }

    static int retAfterTail(int[] iArr) {
        int i = 0;
        for (int i2 : iArr) {
            if (i2 == 3) {
                return i + 100;
            }
            i = i + i2 + 10;
        }
        return i;
    }

    static int labBreak(int[] iArr) {
        int i = 0;
        for (int i2 : iArr) {
            switch (i2) {
                case 5:
                    break;
                default:
                    i += i2;
                    break;
            }
            return i;
        }
        return i;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + retInSwitchNoTail(new int[]{1, 3}) + "/" + retInSwitchTail(new int[]{1, 3}) + "/" + retAfterTail(new int[]{1, 3}) + "/" + labBreak(new int[]{1, 5}));
    }
}

package defpackage;

/* JADX INFO: loaded from: CP7.class */
public class CP7 {
    static int scan(int[] iArr) {
        int i;
        int i2 = 0;
        do {
            i = iArr[i2];
            int i3 = i2;
            i2++;
            if (iArr[i3] == 0) {
                break;
            }
        } while (i2 < iArr.length);
        return i;
    }

    static int find(int[] iArr, int i) {
        int i2 = 0;
        while (i2 < iArr.length) {
            int i3 = i2;
            i2++;
            if (iArr[i3] == i) {
                break;
            }
        }
        return i2 - 1;
    }

    static boolean cond(int[] iArr) {
        return iArr[0] > 0 && 0 + 1 < iArr.length;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + scan(new int[]{5, 7, 0, 9}) + "/" + find(new int[]{5, 7, 9}, 7) + "/" + cond(new int[]{3, 1}));
    }
}

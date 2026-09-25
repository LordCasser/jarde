
/* JADX INFO: loaded from: ForAddStoreLatch.class */
public final class ForAddStoreLatch {
    private ForAddStoreLatch() {
    }

    public static int run(int i, int i2, int i3) {
        int i4 = 0;
        int i5 = 0;
        int i6 = 0;
        while (true) {
            int i7 = i6;
            if (i7 >= i) {
                return i4 + (i5 * 10000);
            }
            i4 += i7;
            int i8 = 0;
            while (true) {
                if (i8 >= 2) {
                    i5++;
                    break;
                }
                if (i7 == i3) {
                    break;
                }
                i8++;
            }
            i6 = i7 + i2;
        }
    }
}

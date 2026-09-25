
/* JADX INFO: loaded from: ForAddStoreSimple.class */
public final class ForAddStoreSimple {
    private ForAddStoreSimple() {
    }

    public static int run(int i, int i2) {
        int i3 = 0;
        int i4 = 0;
        while (true) {
            int i5 = i4;
            if (i5 >= i) {
                return i3;
            }
            i3 += i5;
            i4 = i5 + i2;
        }
    }
}

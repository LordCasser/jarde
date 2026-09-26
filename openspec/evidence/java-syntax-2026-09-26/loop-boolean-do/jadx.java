package defpackage;

/* JADX INFO: loaded from: DoLoopBool.class */
public final class DoLoopBool {
    public static int andDo(int i, int i2) {
        int i3 = 0;
        do {
            i3++;
            i--;
            i2--;
            if (i <= 0) {
                break;
            }
        } while (i2 > 0);
        return i3;
    }

    public static int orDo(int i, int i2) {
        int i3 = 0;
        while (true) {
            i3++;
            i--;
            i2--;
            if (i <= 0 && i2 <= 0) {
                return i3;
            }
        }
    }
}

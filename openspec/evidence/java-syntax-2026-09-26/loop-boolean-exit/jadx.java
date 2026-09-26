package defpackage;

/* JADX INFO: loaded from: LoopBool.class */
public final class LoopBool {
    public static int andWhile(int i, int i2) {
        int i3 = 0;
        while (i > 0 && i2 > 0) {
            i3 += i;
            i--;
            i2--;
        }
        return i3;
    }

    public static int orWhile(int i, int i2) {
        int i3 = 0;
        while (true) {
            if (i <= 0 && i2 <= 0) {
                return i3;
            }
            i3++;
            i--;
            i2--;
        }
    }

    public static int mixedWhile(int i, int i2) {
        int i3 = 0;
        while (true) {
            if ((i <= 0 || i2 <= 0) && i3 >= 2) {
                return i3;
            }
            i3++;
            i--;
            i2--;
        }
    }
}

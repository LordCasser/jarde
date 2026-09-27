package cf08join;

/* JADX INFO: loaded from: LoopIfJoin.jar:cf08join/LoopIfJoin.class */
public final class LoopIfJoin {
    public static int run(boolean z, boolean z2) {
        int i;
        int i2 = 0;
        if (z) {
            if (z2) {
                i2 = 1;
            } else {
                while (i2 < 3) {
                    i2++;
                }
            }
            i = i2 + 10;
        } else {
            i = 4;
        }
        return i;
    }

    public static void main(String[] strArr) {
        System.out.println(run(false, false));
        System.out.println(run(true, false));
        System.out.println(run(true, true));
    }
}

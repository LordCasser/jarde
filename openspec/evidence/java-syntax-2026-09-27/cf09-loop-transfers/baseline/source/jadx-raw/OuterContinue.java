package defpackage;

/* JADX INFO: loaded from: OuterContinue.jar:OuterContinue.class */
final class OuterContinue {
    OuterContinue() {
    }

    static int run(int i) {
        int i2 = 0;
        for (int i3 = 0; i3 < i; i3++) {
            int i4 = 0;
            while (true) {
                if (i4 >= i) {
                    i2 += 100;
                    break;
                }
                if (i4 == 2) {
                    break;
                }
                i2++;
                i4++;
            }
        }
        return i2;
    }
}

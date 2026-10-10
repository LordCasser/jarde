package defpackage;

/* JADX INFO: loaded from: PlainOneArmLoops.class.jar:PlainOneArmLoops.class */
public class PlainOneArmLoops {
    public static int prefixWhile(boolean z, int i) {
        int i2 = 0;
        if (z) {
            for (int i3 = 0; i3 < i; i3++) {
                i2 += i3;
            }
        }
        return i2;
    }

    public static int noPrefix(boolean z, int i) {
        int i2 = 0;
        if (z) {
            while (i2 < i) {
                i2++;
            }
        }
        return i2;
    }

    public static int loopAndTail(boolean z, int i) {
        int i2 = 0;
        if (z) {
            for (int i3 = 0; i3 < i; i3++) {
                i2 += i3;
            }
            i2 += 100;
        }
        return i2;
    }

    public static int takenArm(boolean z, int i) {
        int i2 = 0;
        if (!z) {
            for (int i3 = 0; i3 < i; i3++) {
                i2 += i3;
            }
        }
        return i2;
    }
}

package defpackage;

/* JADX INFO: loaded from: ReuseAfterForEach.class */
public class ReuseAfterForEach {
    public static int sumThenReuse(int[] iArr) {
        int i = 0;
        for (int i2 : iArr) {
            i += i2;
        }
        int i3 = i + 1;
        return i + i3 + i3 + 2;
    }
}

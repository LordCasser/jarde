package cf08;

/* JADX INFO: loaded from: input.jar:cf08/EndlessInts.class */
public final class EndlessInts {
    public static int find(int i) {
        int i2 = 0;
        while (i2 < i && i2 != 3) {
            i2++;
        }
        return i2;
    }
}

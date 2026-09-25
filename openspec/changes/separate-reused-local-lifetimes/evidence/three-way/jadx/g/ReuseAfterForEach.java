package defpackage;

/* JADX INFO: loaded from: ReuseAfterForEach.class */
public class ReuseAfterForEach {
    public static int sumThenReuse(int[] values) {
        int sum = 0;
        for (int value : values) {
            sum += value;
        }
        int first = sum + 1;
        int second = first + 2;
        return sum + first + second;
    }
}

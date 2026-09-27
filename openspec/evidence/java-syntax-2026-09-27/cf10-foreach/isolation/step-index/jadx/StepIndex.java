package defpackage;

/* JADX INFO: loaded from: StepIndex.class */
public class StepIndex {
    public static int everyOther(int[] values) {
        int total = 0;
        for (int i = 0; i < values.length; i += 2) {
            total += values[i];
        }
        return total;
    }

    public static void main(String[] args) {
        System.out.println(everyOther(new int[]{1, 2, 3, 4}));
    }
}

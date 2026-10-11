package defpackage;
public final class ComputedForDirectReturnRunner {
    private static void run(String label, int[] values, int target, int lower) {
        ComputedForDirectReturn.end = values.length;
        ComputedForDirectReturn.calls = 0;
        int result = ComputedForDirectReturn.find(values, target, lower);
        System.out.println(label + ":" + result + ":" + ComputedForDirectReturn.calls);
    }

    public static void main(String[] args) {
        int[] values = {1, 3, 5, 7, 9, 11};
        run("hit", values, 5, 0);
        run("bounded-hit", values, 9, 3);
        run("miss", values, 8, 0);
        run("empty-range", values, 11, 6);
    }
}

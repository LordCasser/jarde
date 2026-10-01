public class C1 {
    interface Op { int apply(int v); }
    public static int captureLocal(int base) {
        final int factor = 3;
        Op op = new Op() {
            @Override public int apply(int v) { return v * factor + base; }
        };
        return op.apply(5);
    }
    public static int captureEffectivelyFinal() {
        int total = 0;
        for (int i = 0; i < 3; i++) {
            final int step = i;
            total += new Op() {
                @Override public int apply(int v) { return v + step; }
            }.apply(step);
        }
        return total;
    }
    public static void main(String[] a) {
        System.out.println(captureLocal(10));
        System.out.println(captureEffectivelyFinal());
    }
}

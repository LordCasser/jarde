public class N2 {
    // Negative 2: 0/1 int from a *non-comparison* source (eager `&` on two booleans lowers to
    // `iand`, no branch arms) at a boolean parameter; stays refused exactly as before the change.
    static String take(boolean f) {
        return f ? "1" : "0";
    }
    public static String ops(boolean p, boolean q) {
        return take(p & q);
    }
    public static void main(String[] a) {
        System.out.println(ops(true, true));
        System.out.println(ops(true, false));
    }
}

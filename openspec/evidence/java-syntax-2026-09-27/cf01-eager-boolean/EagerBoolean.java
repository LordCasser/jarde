public class EagerBoolean {
    static int calls;

    static boolean rhs() {
        calls++;
        return true;
    }

    static void and(boolean left) {
        if ((left & rhs()) != false) {
            calls += 10;
        }
    }

    static void or(boolean left) {
        if ((left | rhs()) != false) {
            calls += 100;
        }
    }

    public static void main(String[] args) {
        and(false);
        System.out.println(calls);
        or(true);
        System.out.println(calls);
    }
}

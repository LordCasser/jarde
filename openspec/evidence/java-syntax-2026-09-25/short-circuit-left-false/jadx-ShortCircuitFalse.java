package defpackage;

/* JADX INFO: loaded from: frozen.jar:ShortCircuitFalse.class */
final class ShortCircuitFalse {
    static boolean result;
    static int calls;

    ShortCircuitFalse() {
    }

    static boolean rhs() {
        calls++;
        return true;
    }

    static void assign(boolean left) {
        result = left && rhs();
    }

    public static void main(String[] args) {
        assign(false);
        System.out.println("false-result=" + result + ",calls=" + calls);
        assign(true);
        System.out.println("true-result=" + result + ",calls=" + calls);
    }
}

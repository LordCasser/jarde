public class SharedTrueDuplicatePhi {
    static boolean result;
    static boolean other;
    static int calls;

    static boolean rhs() {
        calls++;
        return true;
    }

    static void assign(boolean left) {
        result = other = left || rhs();
    }
}

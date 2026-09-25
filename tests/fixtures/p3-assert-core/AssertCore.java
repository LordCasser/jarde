public class AssertCore {
    private static int guardCalls;
    private static int detailCalls;

    private static boolean guard(boolean ok) {
        guardCalls++;
        return ok;
    }

    private static String detail() {
        detailCalls++;
        return "bad";
    }

    public static String check(boolean ok) {
        assert guard(ok) : detail();
        return guardCalls + "|" + detailCalls;
    }

    public static String counts() {
        return guardCalls + "|" + detailCalls;
    }
}

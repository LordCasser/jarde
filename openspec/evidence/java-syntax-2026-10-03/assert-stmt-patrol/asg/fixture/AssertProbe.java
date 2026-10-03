public class AssertProbe {
    private static int guardCalls;
    private static int detailCalls;

    private static boolean guard(int value) {
        guardCalls++;
        return value > 0;
    }

    private static String detail(int value) {
        detailCalls++;
        return "bad:" + value;
    }

    static String check(int value) {
        assert guard(value) : detail(value);
        return guardCalls + "|" + detailCalls;
    }

    public static void main(String[] args) {
        System.out.print(check(1) + ";");
        try {
            System.out.print(check(-1));
        } catch (AssertionError error) {
            System.out.print(error.getMessage() + "|" + guardCalls + "|" + detailCalls);
        }
        System.out.println();
    }
}

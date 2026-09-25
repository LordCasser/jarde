public class FinallyStraightThrowRunner {
    private static void one(boolean failTry, boolean failCleanup) {
        FinallyStraightThrow.trace = 0;
        FinallyStraightThrow.failTry = failTry;
        FinallyStraightThrow.failCleanup = failCleanup;
        try {
            int result = FinallyStraightThrow.run();
            System.out.println(failTry + ":" + failCleanup + ":return=" + result + ":trace=" + FinallyStraightThrow.trace);
        } catch (Throwable failure) {
            System.out.println(failTry + ":" + failCleanup + ":try=" + (failure == FinallyStraightThrow.TRY_FAILURE)
                    + ":cleanup=" + (failure == FinallyStraightThrow.CLEANUP_FAILURE)
                    + ":trace=" + FinallyStraightThrow.trace);
        }
    }

    public static void main(String[] args) {
        one(false, false);
        one(true, false);
        one(false, true);
        one(true, true);
    }
}

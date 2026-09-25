public class ImplicitCleanupRunner {
    private static void run(String name, boolean throwTry, boolean throwCleanup) {
        ImplicitCleanup.trace = 0;
        ImplicitCleanup.throwTry = throwTry;
        ImplicitCleanup.throwCleanup = throwCleanup;
        try {
            int value = ImplicitCleanup.run();
            System.out.println(name + ":return=" + value + ":trace=" + ImplicitCleanup.trace);
        } catch (RuntimeException e) {
            System.out.println(name + ":throw=" + e.getClass().getName()
                    + ":try=" + (e == ImplicitCleanup.TRY_FAILURE)
                    + ":cleanup=" + (e == ImplicitCleanup.CLEANUP_FAILURE)
                    + ":trace=" + ImplicitCleanup.trace);
        }
    }
    public static void main(String[] args) {
        run("normal", false, false);
        run("try-throws", true, false);
        run("cleanup-over-return", false, true);
        run("cleanup-over-try-throw", true, true);
    }
}

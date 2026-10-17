public class FinallyCompletionRunner {
    private static void run(String name, int which) {
        FinallyCompletion.trace = 0;
        try {
            int value;
            switch (which) {
                case 0: value = FinallyCompletion.normalReturn(); break;
                case 1: value = FinallyCompletion.finallyReturns(); break;
                case 2: value = FinallyCompletion.finallyThrows(); break;
                default: value = FinallyCompletion.tryThrowsFinallyRuns(); break;
            }
            System.out.println(name + ":return:" + value + ":" + FinallyCompletion.trace);
        } catch (RuntimeException e) {
            boolean isTry = e == FinallyCompletion.TRY_FAILURE;
            boolean isFinally = e == FinallyCompletion.FINALLY_FAILURE;
            System.out.println(name + ":throw:" + e.getClass().getName() + ":try=" + isTry + ":finally=" + isFinally + ":" + FinallyCompletion.trace);
        }
    }
    public static void main(String[] args) {
        run("normal", 0);
        run("return-overrides", 1);
        run("throw-overrides", 2);
        run("try-throw-preserved", 3);
    }
}

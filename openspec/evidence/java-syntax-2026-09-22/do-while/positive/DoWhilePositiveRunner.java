public class DoWhilePositiveRunner {
    private interface Action {
        Object run();
    }

    private static void run(String name, Action action) {
        DoWhilePositive.reset();
        try {
            System.out.println(name + "=" + action.run() + ":trace=" + DoWhilePositive.trace() +
                ":checks=" + DoWhilePositive.checks());
        } catch (Throwable error) {
            System.out.println(name + "=throws:" + error.getClass().getName() + ":" +
                error.getMessage() + ":trace=" + DoWhilePositive.trace() +
                ":checks=" + DoWhilePositive.checks());
        }
    }

    public static void main(String[] args) {
        run("basic:0", () -> DoWhilePositive.basic(0));
        run("basic:4", () -> DoWhilePositive.basic(4));
        run("effectful:1", () -> DoWhilePositive.effectfulCondition(1));
        run("effectful:3", () -> DoWhilePositive.effectfulCondition(3));
    }
}

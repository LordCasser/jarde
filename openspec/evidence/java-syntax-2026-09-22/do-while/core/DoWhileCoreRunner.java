public class DoWhileCoreRunner {
    private interface Action {
        Object run();
    }

    private static void run(String name, Action action) {
        DoWhileCore.reset();
        try {
            System.out.println(name + "=" + action.run() + ":trace=" + DoWhileCore.trace());
        } catch (Throwable error) {
            System.out.println(name + "=throws:" + error.getClass().getName() + ":" +
                error.getMessage() + ":trace=" + DoWhileCore.trace());
        }
    }

    public static void main(String[] args) {
        run("basic:0", () -> DoWhileCore.basic(0));
        run("basic:1", () -> DoWhileCore.basic(1));
        run("basic:4", () -> DoWhileCore.basic(4));
        run("continue:1", () -> DoWhileCore.withContinue(1));
        run("continue:4", () -> DoWhileCore.withContinue(4));
        run("break:1", () -> DoWhileCore.withBreak(1));
        run("break:5", () -> DoWhileCore.withBreak(5));
    }
}

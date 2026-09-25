public class DoWhileRunner {
    private interface Action {
        Object run();
    }

    private static void observe(String name, Action action) {
        try {
            Object result = action.run();
            System.out.println(name + "=" + result + ":trace=" + DoWhileAudit.trace() +
                ":checks=" + DoWhileAudit.checks());
        } catch (Throwable error) {
            System.out.println(name + "=throws:" + error.getClass().getName() + ":" +
                error.getMessage() + ":trace=" + DoWhileAudit.trace() +
                ":checks=" + DoWhileAudit.checks());
        }
    }

    private static void run(String name, Action action) {
        DoWhileAudit.reset();
        observe(name, action);
    }

    public static void main(String[] args) {
        run("basic:0", () -> DoWhileAudit.basic(0));
        run("basic:1", () -> DoWhileAudit.basic(1));
        run("basic:4", () -> DoWhileAudit.basic(4));
        run("continue:1", () -> DoWhileAudit.withContinue(1));
        run("continue:4", () -> DoWhileAudit.withContinue(4));
        run("break:1", () -> DoWhileAudit.withBreak(1));
        run("break:5", () -> DoWhileAudit.withBreak(5));
        run("effectful:1", () -> DoWhileAudit.effectfulCondition(1));
        run("effectful:3", () -> DoWhileAudit.effectfulCondition(3));
    }
}

public class Runner {
    interface Action { void run() throws Exception; }
    static void observe(String name, Throwable expected, Action action) {
        ThrowEffects.calls = 0;
        try {
            action.run();
            System.out.println(name + "=returned:" + ThrowEffects.calls);
        } catch (Throwable actual) {
            System.out.println(name + "=" + actual.getClass().getName() + ":" +
                (actual == expected) + ":" + ThrowEffects.calls);
        }
    }
    public static void main(String[] args) {
        RuntimeException problem = new IllegalArgumentException("identity");
        RuntimeException second = new IllegalStateException("second");
        java.io.IOException checked = new java.io.IOException("checked");
        ThrowEffects.expected = problem;
        observe("null", null, () -> ThrowAudit.nullValue());
        observe("parameter", problem, () -> ThrowAudit.parameter(problem));
        observe("allocation", null, () -> ThrowAudit.allocation());
        observe("call", problem, () -> ThrowAudit.call());
        observe("cast", problem, () -> ThrowAudit.cast(problem));
        observe("castBad", null, () -> ThrowAudit.cast(new Object()));
        observe("castNull", null, () -> ThrowAudit.cast(null));
        observe("checked", checked, () -> ThrowAudit.checked(checked));
        System.out.println("caught=" + (ThrowAudit.caught(problem) == problem));
        observe("finally", problem, () -> ThrowAudit.withFinally(problem));
        observe("synchronized", problem, () -> ThrowAudit.synchronizedBody(new Object(), problem));
        observe("first", problem, () -> ThrowAudit.conditional(true, problem, second));
        observe("second", second, () -> ThrowAudit.conditional(false, problem, second));
    }
}

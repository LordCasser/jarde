import java.io.IOException;

public final class ThrowProbeRunner {
    private interface Action {
        void run() throws Exception;
    }

    private static void observe(String name, Throwable expected, Action action) {
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
        RuntimeException producerFailure = new IllegalStateException("producer");
        IOException checked = new IOException("checked");
        ThrowEffects.expected = problem;

        observe("null", null, () -> ThrowProbe.nullValue());
        observe("parameter", problem, () -> ThrowProbe.parameter(problem));
        observe("parameterNull", null, () -> ThrowProbe.parameter(null));
        observe("allocation", null, () -> ThrowProbe.allocation());
        observe("call", problem, () -> ThrowProbe.call());
        ThrowEffects.failure = producerFailure;
        observe("callFailure", producerFailure, () -> ThrowProbe.call());
        ThrowEffects.failure = null;
        observe("cast", problem, () -> ThrowProbe.cast(problem));
        observe("castBad", null, () -> ThrowProbe.cast(new Object()));
        observe("castNull", null, () -> ThrowProbe.cast(null));
        observe("first", problem, () -> ThrowProbe.conditional(true, problem, second));
        observe("second", second, () -> ThrowProbe.conditional(false, problem, second));
        observe("checked", checked, () -> ThrowProbe.checked(checked));
        System.out.println("caught=" + (ThrowProbe.namedCatch(problem) == problem));
    }
}

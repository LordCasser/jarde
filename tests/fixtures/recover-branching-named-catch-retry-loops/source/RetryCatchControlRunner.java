package recover.loop2;

import java.security.CodeSource;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.List;

/** Independent, deterministic assertions for the retry handler control. */
public final class RetryCatchControlRunner {
    private RetryCatchControlRunner() {}

    public static void main(String[] args) throws Exception {
        assertTargetCodeSource(args);
        directSuccess();
        retrySuccess();
        rethrowSameException();
        retryEffectFailureEscapesCatch();
        invertedRethrowSameException();
    }

    private static void assertTargetCodeSource(String[] args) throws Exception {
        if (args.length != 1) throw new AssertionError("expected target classes root argument");
        Path expected = Paths.get(args[0]).toRealPath();
        CodeSource source = RetryCatchControl.class.getProtectionDomain().getCodeSource();
        if (source == null || source.getLocation() == null) {
            throw new AssertionError("target CodeSource is unavailable");
        }
        Path actual = Paths.get(source.getLocation().toURI()).toRealPath();
        if (!actual.equals(expected)) {
            throw new AssertionError("target CodeSource: expected " + expected + " but got " + actual);
        }
        System.out.println("TARGET_CODE_SOURCE actual=" + actual + " expected=" + expected);
    }

    private static void directSuccess() throws Exception {
        RetryCatchControl.reset(false, false, false);
        RetryCatchControl.run();
        emitAndAssert("direct-success", list("funcB:1", "return"), 1, 0, false);
    }

    private static void retrySuccess() throws Exception {
        RetryCatchControl.reset(true, false, false);
        RetryCatchControl.run();
        emitAndAssert("retry-success", list("funcB:1", "funcC:B-once:false", "clear",
                "retry-effect", "funcB:2", "return"), 2, 1, false);
        if (RetryCatchControl.saved == null || !"B-once".equals(RetryCatchControl.saved.getMessage())) {
            throw new AssertionError("funcB did not retain its original exception");
        }
    }

    private static void rethrowSameException() throws Exception {
        RetryCatchControl.reset(true, true, false);
        Exception observed = null;
        try {
            RetryCatchControl.run();
        } catch (Exception ex) {
            observed = ex;
        }
        boolean same = observed != null && observed == RetryCatchControl.saved;
        if (!same) throw new AssertionError("rethrow did not preserve the caught exception instance");
        emitAndAssert("rethrow-same-instance", list("funcB:1", "funcC:B-once:true"), 1, 1, same);
    }

    private static void retryEffectFailureEscapesCatch() throws Exception {
        RetryCatchControl.reset(true, false, true);
        IllegalStateException observed = null;
        try {
            RetryCatchControl.run();
        } catch (IllegalStateException ex) {
            observed = ex;
        }
        if (observed == null || !"retry-effect-failed".equals(observed.getMessage())) {
            throw new AssertionError("retryEffect did not escape as the expected exception");
        }
        if (observed == RetryCatchControl.saved) {
            throw new AssertionError("retryEffect failure replaced the saved funcB exception");
        }
        emitAndAssert("retry-effect-fails-outside-catch", list("funcB:1", "funcC:B-once:false",
                "clear", "retry-effect"), 1, 1, false);
    }

    private static void invertedRethrowSameException() throws Exception {
        RetryCatchControl.reset(true, false, false);
        Exception observed = null;
        try {
            RetryCatchControl.runInverted();
        } catch (Exception ex) {
            observed = ex;
        }
        boolean same = observed != null && observed == RetryCatchControl.saved;
        if (!same) throw new AssertionError("inverted branch did not rethrow the same exception instance");
        emitAndAssert("inverted-rethrow-same-instance", list("funcB:1", "funcC:B-once:false"), 1, 1, same);
    }

    private static List<String> list(String... values) {
        return Arrays.asList(values);
    }

    private static void emitAndAssert(String name, List<String> expectedEvents,
            int expectedAttempts, int expectedCatches, boolean sameSaved) {
        List<String> actualEvents = RetryCatchControl.events;
        int actualAttempts = RetryCatchControl.attempts;
        int actualCatches = RetryCatchControl.catches;
        System.out.println("CASE " + name + " events=" + actualEvents + " attempts="
                + actualAttempts + " catches=" + actualCatches + " sameSaved=" + sameSaved);
        if (!actualEvents.equals(expectedEvents)) {
            throw new AssertionError(name + " events: expected " + expectedEvents + " but got " + actualEvents);
        }
        if (actualAttempts != expectedAttempts) {
            throw new AssertionError(name + " attempts: expected " + expectedAttempts + " but got " + actualAttempts);
        }
        if (actualCatches != expectedCatches) {
            throw new AssertionError(name + " catches: expected " + expectedCatches + " but got " + actualCatches);
        }
    }
}

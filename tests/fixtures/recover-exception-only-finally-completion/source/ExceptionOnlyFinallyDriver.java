import java.io.OutputStream;
import java.io.PrintStream;

public class ExceptionOnlyFinallyDriver {
    private static final class RecordingPrintStream extends PrintStream {
        private final StringBuilder events = new StringBuilder();
        private final Throwable tryFailure;
        private final Throwable cleanupFailure;

        RecordingPrintStream(Throwable tryFailure, Throwable cleanupFailure) {
            super(new OutputStream() {
                @Override public void write(int value) { }
            });
            this.tryFailure = tryFailure;
            this.cleanupFailure = cleanupFailure;
        }

        @Override
        public void print(int value) {
            events.append(value);
            Throwable failure = value == 7 ? tryFailure : value == 8 ? cleanupFailure : null;
            if (failure instanceof RuntimeException) throw (RuntimeException) failure;
            if (failure instanceof Error) throw (Error) failure;
            if (failure != null) throw new AssertionError("fixture only injects unchecked failures", failure);
        }
    }

    private static Throwable injected(String kind, String label) {
        switch (kind) {
            case "none": return null;
            case "runtime": return new IllegalStateException(label);
            case "error": return new AssertionError(label);
            default: throw new AssertionError("unknown failure kind: " + kind);
        }
    }

    private static void run(PrintStream savedOut, String tryKind, String cleanupKind) {
        Throwable tryFailure = injected(tryKind, "try-" + tryKind);
        Throwable cleanupFailure = injected(cleanupKind, "cleanup-" + cleanupKind);
        RecordingPrintStream recorder = new RecordingPrintStream(tryFailure, cleanupFailure);
        ExceptionOnlyFinally.trace = 0;
        System.setOut(recorder);
        Throwable actual = null;
        try {
            ExceptionOnlyFinally.tryThrowsFinallyRuns();
        } catch (Throwable failure) {
            actual = failure;
        } finally {
            System.setOut(savedOut);
        }

        Throwable expected = cleanupFailure != null
                ? cleanupFailure
                : tryFailure != null ? tryFailure : ExceptionOnlyFinally.TRY_FAILURE;
        if (actual != expected) {
            throw new AssertionError(tryKind + "/" + cleanupKind + ": exception identity mismatch: " + actual);
        }
        if (!"78".contentEquals(recorder.events)) {
            throw new AssertionError(tryKind + "/" + cleanupKind + ": expected call order/count 78, got " + recorder.events);
        }
        if (ExceptionOnlyFinally.trace != 78) {
            throw new AssertionError(tryKind + "/" + cleanupKind + ": expected trace 78, got " + ExceptionOnlyFinally.trace);
        }
        savedOut.println(tryKind + "/" + cleanupKind
                + ":events=" + recorder.events
                + ":identity=true"
                + ":type=" + actual.getClass().getName()
                + ":message=" + actual.getMessage());
    }

    public static void main(String[] args) {
        PrintStream savedOut = System.out;
        String[] kinds = { "none", "runtime", "error" };
        for (String tryKind : kinds) {
            for (String cleanupKind : kinds) {
                run(savedOut, tryKind, cleanupKind);
            }
        }
    }
}

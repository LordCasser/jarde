import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.util.ArrayList;
import java.util.List;

public class BranchThrowFinallyDriver {
    static final class ObservedOut extends PrintStream {
        final List<String> calls = new ArrayList<String>();
        final Throwable failure;

        ObservedOut(Throwable failure) {
            super(new ByteArrayOutputStream());
            this.failure = failure;
        }

        @Override public void println(String line) {
            calls.add(line);
            if (failure instanceof RuntimeException) throw (RuntimeException) failure;
            if (failure instanceof Error) throw (Error) failure;
        }
    }

    static void check(String label, int n, Throwable cleanupFailure, boolean nullReceiver,
                      String expectedReturn, Class<?> expectedType, String expectedMessage,
                      Throwable expectedIdentity, int expectedCalls) {
        PrintStream saved = System.out;
        ObservedOut observed = nullReceiver ? null : new ObservedOut(cleanupFailure);
        String result = null;
        Throwable thrown = null;
        try {
            System.setOut(observed);
            result = BranchThrowFinally.finThrow(n);
        } catch (Throwable failure) {
            thrown = failure;
        } finally {
            System.setOut(saved);
        }

        if (observed == null) {
            if (expectedCalls != 0) throw new AssertionError(label + ": expected zero observer calls");
        } else if (observed.calls.size() != expectedCalls
                || (expectedCalls == 1 && !"ft".equals(observed.calls.get(0)))) {
            throw new AssertionError(label + ": cleanup calls " + observed.calls);
        }
        if (expectedType == null) {
            if (thrown != null || !expectedReturn.equals(result)) {
                throw new AssertionError(label + ": result=" + result + ", thrown=" + thrown);
            }
        } else {
            if (thrown == null || thrown.getClass() != expectedType) {
                throw new AssertionError(label + ": wrong throwable " + thrown);
            }
            if (expectedMessage != null && !expectedMessage.equals(thrown.getMessage())) {
                throw new AssertionError(label + ": message=" + thrown.getMessage());
            }
            if (expectedIdentity != null && thrown != expectedIdentity) {
                throw new AssertionError(label + ": cleanup throwable identity changed");
            }
        }
        saved.println(label + ":" + (thrown == null ? result : thrown.getClass().getName())
                + ":message=" + (thrown == null ? "" : String.valueOf(thrown.getMessage()))
                + ":calls=" + expectedCalls);
    }

    public static void main(String[] args) {
        RuntimeException cleanupRuntime = new RuntimeException("cleanup-runtime");
        AssertionError cleanupError = new AssertionError("cleanup-error");
        check("zero", 0, null, false, "ok", null, null, null, 1);
        check("positive", 7, null, false, "ok", null, null, null, 1);
        check("primary", -1, null, false, null, RuntimeException.class, "neg", null, 1);
        check("cleanup-runtime", 3, cleanupRuntime, false, null,
                RuntimeException.class, "cleanup-runtime", cleanupRuntime, 1);
        check("cleanup-error", 3, cleanupError, false,
                null, AssertionError.class, "cleanup-error", cleanupError, 1);
        check("primary-cleanup-runtime", -1, cleanupRuntime, false, null,
                RuntimeException.class, "cleanup-runtime", cleanupRuntime, 1);
        check("primary-cleanup-error", -1, cleanupError, false,
                null, AssertionError.class, "cleanup-error", cleanupError, 1);
        check("primary-null-receiver", -1, null, true,
                null, NullPointerException.class, null, null, 0);
        check("null-receiver", 3, null, true,
                null, NullPointerException.class, null, null, 0);
    }
}

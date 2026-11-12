import java.io.ByteArrayOutputStream;
import java.io.PrintStream;

public final class LooplessFinallyContractProbe {
    static final class ObservedOut extends PrintStream {
        final StringBuilder events;
        final RuntimeException failure;
        int calls;
        ObservedOut(StringBuilder events, RuntimeException failure) {
            super(new ByteArrayOutputStream());
            this.events = events;
            this.failure = failure;
        }
        @Override public void println(String text) {
            if (!"f".equals(text)) throw new AssertionError("cleanup argument");
            events.append("cleanup,");
            calls++;
            LooplessFinally.value = 99;
            if (failure != null) throw failure;
        }
    }

    static void check(String name, int[] input, boolean fail, int expected) {
        StringBuilder events = new StringBuilder("before,");
        RuntimeException cause = new RuntimeException("existing-cause");
        RuntimeException previous = new RuntimeException("existing-suppressed");
        RuntimeException cleanup = new RuntimeException("cleanup", cause);
        cleanup.addSuppressed(previous);
        ObservedOut observed = new ObservedOut(events, fail ? cleanup : null);
        PrintStream saved = System.out;
        Throwable thrown = null;
        int result = 777;
        LooplessFinally.value = 3000;
        try {
            System.setOut(observed);
            result = LooplessFinally.loopless(input);
            events.append("returned,");
        } catch (Throwable failure) {
            thrown = failure;
            events.append("thrown,");
        } finally {
            System.setOut(saved);
            events.append("after");
        }
        boolean throwsExpected = fail || input == null;
        String expectedEvents = "before,cleanup," + (throwsExpected ? "thrown," : "returned,") + "after";
        if (result != expected || observed.calls != 1 || LooplessFinally.value != 99
                || !events.toString().equals(expectedEvents))
            throw new AssertionError(name + ": value/count/order");
        if (fail) {
            if (thrown != cleanup || thrown.getCause() != cause)
                throw new AssertionError(name + ": cleanup identity/cause");
            Throwable[] suppressed = thrown.getSuppressed();
            if (suppressed.length != 1 || suppressed[0] != previous)
                throw new AssertionError(name + ": plain finally changed suppression");
        } else if (input == null) {
            if (!(thrown instanceof NullPointerException) || thrown.getSuppressed().length != 0)
                throw new AssertionError(name + ": body failure/suppression");
        } else if (thrown != null) {
            throw new AssertionError(name + ": unexpected failure", thrown);
        }
        saved.println(name + ":result=" + result + ";value=99;calls=1;events=" + events
                + ";throw=" + (fail ? "cleanup;suppressed=existing;cause=existing" :
                    input == null ? "null;suppressed=0" : "none"));
    }

    public static void main(String[] args) {
        check("empty", new int[0], false, -1);
        check("two", new int[]{2}, false, -1);
        check("three", new int[]{3}, false, 3000);
        check("null", null, false, 777);
        check("empty-cleanup", new int[0], true, 777);
        check("two-cleanup", new int[]{2}, true, 777);
        check("return-cleanup", new int[]{3}, true, 777);
        check("null-cleanup", null, true, 777);
    }
}

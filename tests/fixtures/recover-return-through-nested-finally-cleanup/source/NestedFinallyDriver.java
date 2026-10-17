import java.io.ByteArrayOutputStream;
import java.io.PrintStream;

public final class NestedFinallyDriver {
    private NestedFinallyDriver() {
    }

    public static void main(String[] args) {
        for (int n : new int[] { 2, 0 }) {
            for (int inKind = 0; inKind <= 2; inKind++) {
                for (int outKind = 0; outKind <= 2; outKind++) {
                    run(n, inKind, outKind);
                }
            }
        }
        System.out.println("nested-finally: ok");
    }

    private static void run(int n, int inKind, int outKind) {
        Throwable inFailure = failure("in", inKind);
        Throwable outFailure = failure("out", outKind);
        RecordingPrintStream capture = new RecordingPrintStream(inFailure, outFailure);
        PrintStream previous = System.out;
        Integer value = null;
        Throwable escaped = null;
        try {
            System.setOut(capture);
            try {
                value = NestedFinally.nestedFin(n);
            } catch (Throwable failure) {
                escaped = failure;
            }
        } finally {
            System.setOut(previous);
            capture.close();
        }

        if (!capture.events().equals("IO")) {
            throw new AssertionError(label(n, inKind, outKind) + ": events=" + capture.events());
        }
        if (outFailure != null) {
            same(label(n, inKind, outKind), outFailure, escaped);
            if (value != null) {
                throw new AssertionError("thrown completion also returned a value");
            }
        } else if (inFailure != null) {
            same(label(n, inKind, outKind), inFailure, escaped);
            if (value != null) {
                throw new AssertionError("thrown completion also returned a value");
            }
        } else if (n == 0) {
            if (!(escaped instanceof ArithmeticException) || value != null) {
                throw new AssertionError(label(n, inKind, outKind) + ": escaped=" + escaped);
            }
            System.out.println(label(n, inKind, outKind) + ": throw="
                    + escaped.getClass().getName() + ":message=" + escaped.getMessage()
                    + ":events=" + capture.events());
        } else {
            if (escaped != null || value == null || value.intValue() != 5) {
                throw new AssertionError(label(n, inKind, outKind) + ": value=" + value
                        + " escaped=" + escaped);
            }
            System.out.println(label(n, inKind, outKind) + ": return=" + value
                    + ":events=" + capture.events());
        }
    }

    private static Throwable failure(String where, int kind) {
        if (kind == 1) {
            return new RuntimeException(where + "-runtime");
        }
        if (kind == 2) {
            return new AssertionError(where + "-error");
        }
        return null;
    }

    private static String label(int n, int inKind, int outKind) {
        return "n=" + n + ",in=" + inKind + ",out=" + outKind;
    }

    private static void same(String label, Throwable expected, Throwable actual) {
        if (actual != expected) {
            throw new AssertionError(label + ": escaping object identity differs; actual=" + actual);
        }
        System.out.println(label + ": throw=" + actual.getClass().getName() + ":message=" + actual.getMessage()
                + ":sameInjectedObject=true:events=IO");
    }

    private static final class RecordingPrintStream extends PrintStream {
        private final Throwable inFailure;
        private final Throwable outFailure;
        private final StringBuilder events = new StringBuilder();

        RecordingPrintStream(Throwable inFailure, Throwable outFailure) {
            super(new ByteArrayOutputStream());
            this.inFailure = inFailure;
            this.outFailure = outFailure;
        }

        @Override
        public void println(String value) {
            if ("in".equals(value)) {
                events.append('I');
                throwInjected(inFailure);
            } else if ("out".equals(value)) {
                events.append('O');
                throwInjected(outFailure);
            } else {
                throw new AssertionError("unexpected println argument: " + value);
            }
        }

        String events() {
            return events.toString();
        }

        private static void throwInjected(Throwable failure) {
            if (failure instanceof RuntimeException) {
                throw (RuntimeException) failure;
            }
            if (failure instanceof Error) {
                throw (Error) failure;
            }
        }
    }
}

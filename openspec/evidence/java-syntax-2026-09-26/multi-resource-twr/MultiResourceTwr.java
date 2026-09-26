/** Two-resource TWR fixture with independently selectable body and close failures. */
public final class MultiResourceTwr {
    private static final StringBuilder EVENTS = new StringBuilder();
    private static boolean bodyFails;
    private static boolean innerCloseFails;
    private static boolean outerCloseFails;

    private static final class Probe implements AutoCloseable {
        private final String name;
        Probe(String name) {
            this.name = name;
            event("open-" + name);
        }

        int read() {
            event("read");
            return 42;
        }

        @Override
        public void close() throws Exception {
            event("close-" + name);
            if ((name.equals("inner") && innerCloseFails)
                    || (name.equals("outer") && outerCloseFails)) {
                throw new CloseFailure(name);
            }
        }
    }

    private static final class BodyFailure extends Exception {
        BodyFailure() { super("body"); }
    }

    private static final class CloseFailure extends Exception {
        CloseFailure(String name) { super("close-" + name); }
    }

    private static void event(String value) {
        if (EVENTS.length() != 0) EVENTS.append(',');
        EVENTS.append(value);
    }

    private static void maybeFailBody() throws BodyFailure {
        if (bodyFails) throw new BodyFailure();
    }

    private static int run() throws Exception {
        try (Probe outer = new Probe("outer");
             Probe inner = new Probe("inner")) {
            event("body");
            maybeFailBody();
            return inner.read();
        }
    }

    public static void main(String[] args) {
        String mode = args.length == 0 ? "normal" : args[0];
        EVENTS.setLength(0);
        try {
            bodyFails = mode.equals("body") || mode.equals("suppressed");
            innerCloseFails = mode.equals("inner-close") || mode.equals("suppressed");
            outerCloseFails = mode.equals("outer-close") || mode.equals("suppressed");
            int result = run();
            System.out.println("result=" + result + ";events=" + EVENTS);
        } catch (Throwable failure) {
            StringBuilder suppressed = new StringBuilder();
            for (Throwable item : failure.getSuppressed()) {
                if (suppressed.length() != 0) suppressed.append(',');
                suppressed.append(item.getClass().getSimpleName()).append(':').append(item.getMessage());
            }
            System.out.println("throw=" + failure.getClass().getSimpleName() + ':' + failure.getMessage()
                    + ";suppressed=" + suppressed + ";events=" + EVENTS);
        }
    }
}

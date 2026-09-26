package defpackage;

/* JADX INFO: loaded from: MultiResourceTwr.class */
public final class MultiResourceTwr {
    private static final StringBuilder EVENTS = new StringBuilder();
    private static boolean bodyFails;
    private static boolean innerCloseFails;
    private static boolean outerCloseFails;

    /* JADX INFO: loaded from: MultiResourceTwr$BodyFailure.class */
    private static final class BodyFailure extends Exception {
        BodyFailure() {
            super("body");
        }
    }

    /* JADX INFO: loaded from: MultiResourceTwr$CloseFailure.class */
    private static final class CloseFailure extends Exception {
        CloseFailure(String str) {
            super("close-" + str);
        }
    }

    /* JADX INFO: loaded from: MultiResourceTwr$Probe.class */
    private static final class Probe implements AutoCloseable {
        private final String name;

        Probe(String str) {
            this.name = str;
            MultiResourceTwr.event("open-" + str);
        }

        int read() {
            MultiResourceTwr.event("read");
            return 42;
        }

        @Override // java.lang.AutoCloseable
        public void close() throws Exception {
            MultiResourceTwr.event("close-" + this.name);
            if ((this.name.equals("inner") && MultiResourceTwr.innerCloseFails) || (this.name.equals("outer") && MultiResourceTwr.outerCloseFails)) {
                throw new CloseFailure(this.name);
            }
        }
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static void event(String str) {
        if (EVENTS.length() != 0) {
            EVENTS.append(',');
        }
        EVENTS.append(str);
    }

    private static void maybeFailBody() throws BodyFailure {
        if (bodyFails) {
            throw new BodyFailure();
        }
    }

    private static int run() throws Exception {
        Probe probe = new Probe("outer");
        try {
            Probe probe2 = new Probe("inner");
            try {
                event("body");
                maybeFailBody();
                int i = probe2.read();
                probe2.close();
                probe.close();
                return i;
            } catch (Throwable th) {
                try {
                    probe2.close();
                } catch (Throwable th2) {
                    th.addSuppressed(th2);
                }
                throw th;
            }
        } catch (Throwable th3) {
            try {
                probe.close();
            } catch (Throwable th4) {
                th3.addSuppressed(th4);
            }
            throw th3;
        }
    }

    public static void main(String[] strArr) {
        String str = strArr.length == 0 ? "normal" : strArr[0];
        EVENTS.setLength(0);
        try {
            bodyFails = str.equals("body") || str.equals("suppressed");
            innerCloseFails = str.equals("inner-close") || str.equals("suppressed");
            outerCloseFails = str.equals("outer-close") || str.equals("suppressed");
            System.out.println("result=" + run() + ";events=" + ((Object) EVENTS));
        } catch (Throwable th) {
            StringBuilder sb = new StringBuilder();
            for (Throwable th2 : th.getSuppressed()) {
                if (sb.length() != 0) {
                    sb.append(',');
                }
                sb.append(th2.getClass().getSimpleName()).append(':').append(th2.getMessage());
            }
            System.out.println("throw=" + th.getClass().getSimpleName() + ':' + th.getMessage() + ";suppressed=" + ((Object) sb) + ";events=" + ((Object) EVENTS));
        }
    }
}

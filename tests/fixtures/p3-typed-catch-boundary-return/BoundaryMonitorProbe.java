public final class BoundaryMonitorProbe {
    static final Object LOCK = new Object();
    static int calls;

    static void touch() {
        calls++;
    }

    static synchronized String synchronizedMethod(int mode) {
        try {
            if (mode == 1) throw new IllegalArgumentException("a");
            if (mode == 2) throw new IllegalStateException("s");
            return "ok";
        } catch (IllegalArgumentException | IllegalStateException error) {
            return error.getClass().getSimpleName() + ":" + error.getMessage();
        }
    }

    static String explicitMonitor(int mode) {
        synchronized (LOCK) {
            touch();
        }
        try {
            if (mode == 1) throw new IllegalArgumentException("a");
            if (mode == 2) throw new IllegalStateException("s");
            return "ok";
        } catch (IllegalArgumentException | IllegalStateException error) {
            return error.getClass().getSimpleName() + ":" + error.getMessage();
        }
    }

    public static void main(String[] args) {
        System.out.println(synchronizedMethod(0));
        System.out.println(explicitMonitor(0));
        System.out.println("calls=" + calls);
    }
}

public class W17c implements AutoCloseable {
    static StringBuilder log = new StringBuilder();
    static boolean closeBoom = false;

    @Override public void close() {
        if (closeBoom) {
            throw new IllegalStateException("close");
        }
    }

    static void touch(W17c r) { }

    static void boom() { throw new IllegalStateException("body"); }

    public static String normalReturn() {
        try (W17c r = new W17c()) { touch(r); } catch (IllegalStateException e) { return "caught"; }
        return "done";
    }

    public static String callCatch() {
        try (W17c r = new W17c()) { touch(r); } catch (IllegalStateException e) { log.append("E"); }
        return "done";
    }

    public static String bodyThrows() {
        try (W17c r = new W17c()) { boom(); } catch (IllegalStateException e) { log.append("E"); }
        return "done";
    }

    public static String callThenReturn() {
        try (W17c r = new W17c()) { boom(); } catch (IllegalStateException e) { log.append("E"); return "caught"; }
        return "done";
    }

    public static String twoCalls() {
        try (W17c r = new W17c()) { boom(); } catch (IllegalStateException e) { log.append("a"); log.append("b"); }
        return "done";
    }

    public static String chainedConsume() {
        try (W17c r = new W17c()) { boom(); } catch (IllegalStateException e) { log.append(e.getMessage()); }
        return "done";
    }

    public static String branchBody() {
        try (W17c r = new W17c()) { boom(); } catch (IllegalStateException e) { log.append("B"); if (closeBoom) { log.append("1"); } else { log.append("2"); } }
        return "done";
    }

    public static String closeThrows() {
        try (W17c r = new W17c()) { touch(r); } catch (IllegalStateException e) { log.append("C"); }
        return "done";
    }

    public static String suppressedBoth() {
        try (W17c r = new W17c()) { boom(); } catch (IllegalStateException e) { log.append("[" + java.util.Arrays.toString(e.getSuppressed()) + "]"); }
        return "done";
    }

    public static void main(String[] args) {
        System.out.println(normalReturn());
        System.out.println(callCatch());
        System.out.println(bodyThrows());
        System.out.println(callThenReturn());
        System.out.println(twoCalls());
        System.out.println(chainedConsume());
        closeBoom = true;
        System.out.println(closeThrows());
        System.out.println(suppressedBoth());
        closeBoom = false;
        System.out.println(branchBody());
        System.out.println("log:" + log);
    }
}

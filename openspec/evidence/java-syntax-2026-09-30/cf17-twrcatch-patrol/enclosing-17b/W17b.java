public class W17b implements AutoCloseable {
    static boolean closeBoom = false;

    @Override public void close() {
        if (closeBoom) {
            throw new IllegalStateException("close");
        }
    }

    static void touch(W17b r) { }

    static void boom() {
        throw new IllegalStateException("body");
    }

    public static String normalReturn() {
        try (W17b r = new W17b()) { touch(r); } catch (IllegalStateException e) { return "caught"; }
        return "done";
    }

    public static String bodyThrows() {
        try (W17b r = new W17b()) { boom(); } catch (IllegalStateException e) { return "caught:" + e.getMessage(); }
        return "done";
    }

    public static String handlerCall() {
        try (W17b r = new W17b()) { boom(); } catch (IllegalStateException e) { touch(null); }
        return "done";
    }

    public static String closeThrows() {
        try (W17b r = new W17b()) { touch(r); } catch (IllegalStateException e) { return "caught:" + e.getMessage() + ":" + java.util.Arrays.toString(e.getSuppressed()); }
        return "done";
    }

    public static String suppressedBoth() {
        try (W17b r = new W17b()) { boom(); } catch (IllegalStateException e) { return "caught:" + e.getMessage() + ":" + java.util.Arrays.toString(e.getSuppressed()); }
        return "done";
    }

    public static void main(String[] args) {
        System.out.println(normalReturn());
        System.out.println(bodyThrows());
        System.out.println(handlerCall());
        closeBoom = true;
        System.out.println(closeThrows());
        System.out.println(suppressedBoth());
        closeBoom = false;
    }
}

public final class MultiCatchProbe {
    static int finallyCalls;
    static String choosePlain(int mode) {
        try {
            if (mode == 1) throw new IllegalArgumentException("a");
            if (mode == 2) throw new IllegalStateException("s");
            return "ok";
        } catch (IllegalArgumentException | IllegalStateException ex) {
            return ex.getClass().getSimpleName() + ":" + ex.getMessage();
        }
    }
    static String chooseFinally(int mode) {
        try {
            if (mode == 1) throw new IllegalArgumentException("a");
            if (mode == 2) throw new IllegalStateException("s");
            return "ok";
        } catch (IllegalArgumentException | IllegalStateException ex) {
            return ex.getClass().getSimpleName() + ":" + ex.getMessage();
        } finally {
            finallyCalls++;
        }
    }
}

public final class PlainMultiCatch {
    static String choose(int mode) {
        try {
            if (mode == 1) throw new IllegalArgumentException("a");
            if (mode == 2) throw new IllegalStateException("s");
            return "ok";
        } catch (IllegalArgumentException | IllegalStateException ex) {
            return ex.getClass().getSimpleName() + ":" + ex.getMessage();
        }
    }
}

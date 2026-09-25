public final class BoundaryProbe {
    public static boolean third(boolean a, boolean b) {
        if (a) return true;
        if (b) return false;
        return true;
    }

    public static boolean backedge(int count) {
        while (count > 0) {
            if (count == 1) return true;
            count--;
        }
        return false;
    }

    public static boolean exception(String value) {
        try {
            if (value.equals("x")) return true;
        } catch (RuntimeException ignored) {
            return false;
        }
        return false;
    }
}

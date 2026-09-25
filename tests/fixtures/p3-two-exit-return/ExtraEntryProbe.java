public final class ExtraEntryProbe {
    public static boolean check(boolean first, boolean second, boolean third) {
        if (first) System.nanoTime();
        if (second || third) return true;
        return false;
    }
}

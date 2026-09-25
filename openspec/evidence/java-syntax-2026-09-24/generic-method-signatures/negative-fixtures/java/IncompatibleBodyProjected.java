public class IncompatibleBodyProbe {
    public static <T extends Number> T choose(T left, T right, boolean first) {
        return Integer.valueOf(7);
    }
}

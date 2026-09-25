public class GenericMethodProbe {
    public static <T extends Number> T choose(T left, T right, boolean first) {
        return first ? left : right;
    }
}

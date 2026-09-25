public final class ArrayPostfixElement {
    static int[] make(int a) {
        return new int[] { 1, a++, a * 2 };
    }
}

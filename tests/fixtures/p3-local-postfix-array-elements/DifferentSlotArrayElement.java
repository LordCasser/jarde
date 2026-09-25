public final class DifferentSlotArrayElement {
    static int[] make(int a, int b) {
        return new int[] { 1, a++, a * 2 };
    }
}

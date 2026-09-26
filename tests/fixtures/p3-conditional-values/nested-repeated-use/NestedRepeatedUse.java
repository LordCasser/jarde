final class NestedRepeatedUse {
    static void sink(int first, int second) {}

    static void run(int value) {
        sink(value > 10 ? (value > 100 ? 3 : 2) : 1, value > 0 ? 4 : 5);
    }
}

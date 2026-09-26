final class NestedTree {
    static int falseNested(int value) {
        return value > 10 ? 1 : (value > 0 ? 2 : (value > -10 ? 3 : 4));
    }
}

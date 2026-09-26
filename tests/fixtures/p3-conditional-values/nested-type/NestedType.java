final class NestedType {
    static final class Left {}
    static final class Right {}

    static Object unknown(int value, Left left, Right right) {
        return value > 10 ? (value > 0 ? left : right) : left;
    }
}

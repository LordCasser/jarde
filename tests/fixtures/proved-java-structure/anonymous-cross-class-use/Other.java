final class Other {
    static Base two() {
        return new Base() {
            int value() { return 2; }
        };
    }
}

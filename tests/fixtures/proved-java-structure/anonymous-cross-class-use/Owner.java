final class Owner {
    static Base one() {
        return new Base() {
            int value() { return 1; }
        };
    }
}

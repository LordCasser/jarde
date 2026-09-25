public final class CarriedNonPrologue {
    private final Nested value;

    public CarriedNonPrologue(String input, int mode) {
        this.value = new Nested(mode == 1 ? input : "", mode == 0 ? "zero" : "other");
    }

    public String values() {
        return value.first + ":" + value.second;
    }

    private static final class Nested {
        private final String first;
        private final String second;

        Nested(String first, String second) {
            this.first = first;
            this.second = second;
        }
    }
}

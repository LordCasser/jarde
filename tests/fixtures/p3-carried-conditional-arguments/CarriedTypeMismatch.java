public final class CarriedTypeMismatch {
    private final Object first;
    private final String second;

    public CarriedTypeMismatch(Object input, boolean select, int mode) {
        this(select ? input : "", mode == 0 ? "zero" : "other");
    }

    private CarriedTypeMismatch(Object first, String second) {
        this.first = first;
        this.second = second;
    }

    public String values() {
        return first + ":" + second;
    }
}

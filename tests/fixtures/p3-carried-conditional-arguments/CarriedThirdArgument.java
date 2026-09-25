public final class CarriedThirdArgument {
    private final String first;
    private final String second;
    private final int third;

    public CarriedThirdArgument(String input, int mode) {
        this(mode == 1 ? input : "", mode == 0 ? "zero" : "other", mode);
    }

    private CarriedThirdArgument(String first, String second, int third) {
        this.first = first;
        this.second = second;
        this.third = third;
    }

    public String values() {
        return first + ":" + second + ":" + third;
    }
}

public final class CarriedInterveningEffect {
    private static final StringBuilder TRACE = new StringBuilder();
    private final String first;
    private final String middle;
    private final String second;

    private static String touch() {
        TRACE.append("X");
        return "middle";
    }

    public CarriedInterveningEffect(String input, int mode) {
        this(mode == 1 ? input : "", touch(), mode == 0 ? "zero" : "other");
    }

    private CarriedInterveningEffect(String first, String middle, String second) {
        this.first = first;
        this.middle = middle;
        this.second = second;
    }

    public String values() {
        return TRACE.toString() + ":" + first + ":" + middle + ":" + second;
    }
}

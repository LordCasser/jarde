package dt29p2;

interface ExtraBound {}

public class OverloadedSetter {
    private static void choose(Bound value) {}

    private static void choose(ExtraBound value) {}

    public void set(Bound value, boolean enabled) {
        choose(value);
    }
}

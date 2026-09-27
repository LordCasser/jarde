package dt29p2;

public class Bound {
    private int calls;

    public Bound self(boolean enabled) {
        if (enabled) {
            calls++;
        }
        return this;
    }

    public int calls() {
        return calls;
    }
}

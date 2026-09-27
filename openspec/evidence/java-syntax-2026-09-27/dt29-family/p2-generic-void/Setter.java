package dt29p2;

public class Setter {
    public <T extends Bound> void set(T value, boolean enabled) {
        value.self(enabled);
    }

    public void set(Bound value, boolean enabled, int marker) {
        value.self(enabled);
    }
}

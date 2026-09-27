package dt29p2;

public class IncompatibleSetter {
    public void set(Bound value, boolean enabled) {
        value = value.self(enabled);
    }
}

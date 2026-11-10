package bounds;

public final class Bounded<T extends String> {
    public <T> T shadow(T value) {
        return value;
    }
}

package classvars;

public final class MethodShadowBoundary<T> {
    public <T> T identity(T value) {
        return value;
    }
}

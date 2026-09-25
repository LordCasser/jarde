package classvars;

public final class OuterScopeBoundary<T> {
    public final class Inner {
        public T identity(T value) {
            return value;
        }
    }
}

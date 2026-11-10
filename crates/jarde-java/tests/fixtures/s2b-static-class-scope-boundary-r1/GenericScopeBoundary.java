package scope;

public class GenericScopeBoundary<T> {
    public static Object pass(Object value) {
        Object copy = value;
        return copy;
    }

}

class GenericScopeShadow<T> {
    public static <T> T pass(T value) {
        T copy = value;
        return copy;
    }
}

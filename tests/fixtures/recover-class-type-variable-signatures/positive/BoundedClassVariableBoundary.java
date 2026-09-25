package classvars;

public final class BoundedClassVariableBoundary<U extends Number & Comparable<U>> {
    public U identity(U value) {
        return value;
    }
}

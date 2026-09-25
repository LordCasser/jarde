package fieldsig;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.lang.reflect.Array;

public final class FieldSignatureBoundary<T> {
    public List<String> names = new ArrayList<String>();
    private final List<? extends Number> numbers = Arrays.<Number>asList(7, 11);
    public T current;
    public T[] vector;
    public List<? super T> sink = new ArrayList<Object>();
    public static final int TOKEN = 41;
    public static final String LABEL = "field-signature";

    @SuppressWarnings("unchecked")
    public FieldSignatureBoundary(T current) {
        this.current = current;
        Class<?> component = current == null ? Object.class : current.getClass();
        this.vector = (T[]) Array.newInstance(component, 1);
        this.vector[0] = current;
    }

    public List<? extends Number> readNumbers() {
        return numbers;
    }
}

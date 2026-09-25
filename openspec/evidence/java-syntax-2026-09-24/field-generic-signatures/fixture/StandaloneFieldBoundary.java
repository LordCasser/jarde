package fieldsig;

import java.util.List;

public final class StandaloneFieldBoundary<T> {
    public List<String> names;
    private List<? extends Number> numbers;
    public T current;
    public T[] vector;
    protected List<? super T> sink;
    public static final int TOKEN = 41;
    public static final String LABEL = "field-signature";
}

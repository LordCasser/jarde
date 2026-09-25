import java.util.Iterator;

public final class NamedIteratorCursor {
    private final Iterator<String> delegate;

    public NamedIteratorCursor(Iterator<String> delegate) {
        this.delegate = delegate;
    }

    public Iterator<String> iterator() {
        return delegate;
    }

    public static int sum(NamedIteratorCursor cursor) {
        Iterator<String> it = cursor.iterator();
        int result = 0;
        while (it.hasNext()) {
            String value = it.next();
            result += value.length();
        }
        return result;
    }
}

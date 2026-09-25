package defpackage;

import java.util.Iterator;

/* JADX INFO: loaded from: NamedIteratorCursor.class */
public final class NamedIteratorCursor {
    private final Iterator<String> delegate;

    public NamedIteratorCursor(Iterator<String> delegate) {
        this.delegate = delegate;
    }

    public Iterator<String> iterator() {
        return this.delegate;
    }

    public static int sum(NamedIteratorCursor cursor) {
        Iterator<String> it = cursor.iterator();
        int length = 0;
        while (true) {
            int result = length;
            if (it.hasNext()) {
                String value = it.next();
                length = result + value.length();
            } else {
                return result;
            }
        }
    }
}

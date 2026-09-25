package defpackage;

import java.util.Iterator;

/* JADX INFO: loaded from: NamedIteratorCursor.class */
public final class NamedIteratorCursor {
    private final Iterator<String> delegate;

    public NamedIteratorCursor(Iterator<String> it) {
        this.delegate = it;
    }

    public Iterator<String> iterator() {
        return this.delegate;
    }

    public static int sum(NamedIteratorCursor namedIteratorCursor) {
        Iterator<String> it = namedIteratorCursor.iterator();
        int length = 0;
        while (true) {
            int i = length;
            if (!it.hasNext()) {
                return i;
            }
            length = i + it.next().length();
        }
    }
}

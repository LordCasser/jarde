package defpackage;

import java.util.Iterator;

/* JADX INFO: loaded from: CatchNextScope.class */
public final class CatchNextScope {
    public static int consume(Iterable iterable) {
        Iterator it = iterable.iterator();
        int length = 0;
        while (it.hasNext()) {
            try {
                length += ((String) it.next()).length();
            } catch (IllegalStateException e) {
                length++;
            }
        }
        return length;
    }
}

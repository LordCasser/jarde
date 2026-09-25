package defpackage;

import java.util.Iterator;

/* JADX INFO: loaded from: IterableForEach.class */
public final class IterableForEach {
    public String test(Iterable<String> iterable) {
        StringBuilder sb = new StringBuilder();
        Iterator<String> it = iterable.iterator();
        while (it.hasNext()) {
            sb.append(it.next());
        }
        return sb.toString();
    }
}

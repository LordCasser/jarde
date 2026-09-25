package defpackage;

import java.util.Collection;
import java.util.Iterator;
import java.util.List;

/* JADX INFO: loaded from: SubtypeOwners.class */
public final class SubtypeOwners {
    public static int sumList(List<String> list) {
        int length = 0;
        Iterator<String> it = list.iterator();
        while (it.hasNext()) {
            length += it.next().length();
        }
        return length;
    }

    public static int sumCollection(Collection<String> collection) {
        int length = 0;
        Iterator<String> it = collection.iterator();
        while (it.hasNext()) {
            length += it.next().length();
        }
        return length;
    }

    public static int sumTextIterable(TextIterable textIterable) {
        int length = 0;
        Iterator it = textIterable.iterator();
        while (it.hasNext()) {
            length += ((String) it.next()).length();
        }
        return length;
    }
}

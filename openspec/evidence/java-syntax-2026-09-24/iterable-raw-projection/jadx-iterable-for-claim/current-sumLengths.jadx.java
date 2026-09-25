package defpackage;

import java.util.Iterator;
import java.util.function.Supplier;

/* JADX INFO: loaded from: StringIterableForeach.class */
final class StringIterableForeach {
    StringIterableForeach() {
    }

    static int sumLengths(Iterable<String> iterable) {
        int length = 0;
        Iterator<String> it = iterable.iterator();
        while (it.hasNext()) {
            length += it.next().length();
        }
        return length;
    }

    static int sumLengthsFrom(Supplier<Iterable<String>> supplier) {
        int length = 0;
        Iterator<String> it = supplier.get().iterator();
        while (it.hasNext()) {
            length += it.next().length();
        }
        return length;
    }
}

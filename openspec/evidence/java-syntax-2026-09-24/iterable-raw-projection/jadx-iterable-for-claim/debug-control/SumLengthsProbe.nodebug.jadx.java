package defpackage;

import java.util.Iterator;

/* JADX INFO: loaded from: SumLengthsProbe.class */
public final class SumLengthsProbe {
    public static int sumLengths(Iterable<String> iterable) {
        int length = 0;
        Iterator<String> it = iterable.iterator();
        while (it.hasNext()) {
            length += it.next().length();
        }
        return length;
    }
}

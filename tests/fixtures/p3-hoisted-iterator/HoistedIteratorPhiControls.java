package cf11;

import java.util.Iterator;
import java.util.List;

public final class HoistedIteratorPhiControls {
    public static int mixedIterator(List a, List b, boolean flag) {
        Iterator it = a.iterator();
        if (flag) {
            it = b.iterator();
        }
        int seen = 0;
        while (it.hasNext()) {
            it.next();
            seen++;
        }
        return seen;
    }

    public static boolean extraAliasConsumer(List a, List b, boolean flag) {
        Iterator original = a.iterator();
        Iterator alias;
        if (flag) {
            alias = original;
        } else {
            alias = b.iterator();
        }
        return alias.hasNext();
    }
}

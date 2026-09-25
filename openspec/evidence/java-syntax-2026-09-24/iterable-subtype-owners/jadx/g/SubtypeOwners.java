package defpackage;

import java.util.Collection;
import java.util.Iterator;
import java.util.List;

/* JADX INFO: loaded from: SubtypeOwners.class */
public final class SubtypeOwners {
    public static int sumList(List<String> values) {
        int sum = 0;
        for (String value : values) {
            sum += value.length();
        }
        return sum;
    }

    public static int sumCollection(Collection<String> values) {
        int sum = 0;
        for (String value : values) {
            sum += value.length();
        }
        return sum;
    }

    public static int sumTextIterable(TextIterable values) {
        int sum = 0;
        Iterator it = values.iterator();
        while (it.hasNext()) {
            String value = (String) it.next();
            sum += value.length();
        }
        return sum;
    }
}

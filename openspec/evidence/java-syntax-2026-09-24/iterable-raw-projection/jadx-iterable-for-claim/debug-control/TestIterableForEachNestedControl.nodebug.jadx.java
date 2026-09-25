package defpackage;

import java.util.Iterator;

/* JADX INFO: loaded from: nodebug.jar:TestIterableForEach.class */
public final class TestIterableForEach {

    /* JADX INFO: loaded from: nodebug.jar:TestIterableForEach$TestCls.class */
    public static class TestCls {
        public String test(Iterable<String> iterable) {
            StringBuilder sb = new StringBuilder();
            Iterator<String> it = iterable.iterator();
            while (it.hasNext()) {
                sb.append(it.next());
            }
            return sb.toString();
        }
    }
}

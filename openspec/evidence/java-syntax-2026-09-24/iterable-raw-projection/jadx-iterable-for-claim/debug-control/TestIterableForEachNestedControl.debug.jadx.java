package defpackage;

/* JADX INFO: loaded from: debug.jar:TestIterableForEach.class */
public final class TestIterableForEach {

    /* JADX INFO: loaded from: debug.jar:TestIterableForEach$TestCls.class */
    public static class TestCls {
        public String test(Iterable<String> a) {
            StringBuilder sb = new StringBuilder();
            for (String s : a) {
                sb.append(s);
            }
            return sb.toString();
        }
    }
}

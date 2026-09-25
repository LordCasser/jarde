package defpackage;

/* JADX INFO: loaded from: CatchNextScope.class */
public final class CatchNextScope {
    public static int consume(Iterable values) {
        int result = 0;
        for (Object item : values) {
            try {
                String text = (String) item;
                result += text.length();
            } catch (IllegalStateException e) {
                result++;
            }
        }
        return result;
    }
}

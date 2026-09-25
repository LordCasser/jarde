package defpackage;

/* JADX INFO: loaded from: PlainMultiCatch.class */
public final class PlainMultiCatch {
    static java.lang.String choose(int i) {
        try {
            if (i == 1) {
                throw new java.lang.IllegalArgumentException("a");
            }
            if (i == 2) {
                throw new java.lang.IllegalStateException("s");
            }
            return "ok";
        } catch (java.lang.IllegalArgumentException | java.lang.IllegalStateException e) {
            return e.getClass().getSimpleName() + ":" + e.getMessage();
        }
    }
}

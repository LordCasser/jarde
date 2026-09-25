package defpackage;

/* JADX INFO: loaded from: ExceptionProbe.class */
public interface ExceptionProbe {
    public static final String FIRST = BoundaryEffects.first();
    public static final String SECOND = BoundaryEffects.second();

    static String observe() {
        return BoundaryEffects.trace + "|" + FIRST + "|" + SECOND;
    }

    static {
        try {
            BoundaryEffects.fail();
        } catch (IllegalStateException e) {
            BoundaryEffects.caught();
        }
    }
}

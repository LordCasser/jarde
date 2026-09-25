package defpackage;

/* JADX INFO: loaded from: ForwardProbe.class */
public interface ForwardProbe {
    public static final int LATE = BoundaryEffects.value();
    public static final int EARLY = LATE;

    static String observe() {
        return BoundaryEffects.trace + "|" + EARLY + "|" + LATE;
    }
}

package defpackage;

/* JADX INFO: loaded from: BoundaryProbe.class */
public interface BoundaryProbe {
    public static final String FIRST = null;
    public static final String SECOND;

    static String observe() {
        return BoundaryEffects.trace + "|" + FIRST + "|" + SECOND;
    }

    static void patchAnchor() {
        BoundaryEffects.independent();
    }

    static {
        SECOND = BoundaryEffects.next("A");
        SECOND = BoundaryEffects.next("B");
    }
}

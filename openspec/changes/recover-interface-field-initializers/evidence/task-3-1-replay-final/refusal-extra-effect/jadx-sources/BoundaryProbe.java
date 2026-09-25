package defpackage;

/* JADX INFO: loaded from: BoundaryProbe.class */
public interface BoundaryProbe {
    public static final String FIRST = BoundaryEffects.next("A");
    public static final String SECOND = BoundaryEffects.next("B");

    static String observe() {
        return BoundaryEffects.trace + "|" + FIRST + "|" + SECOND;
    }

    static void patchAnchor() {
        BoundaryEffects.independent();
    }

    static {
        BoundaryEffects.independent();
    }
}

package defpackage;

/* JADX INFO: loaded from: BoundaryProbe.class */
public interface BoundaryProbe {
    public static final String FIRST;
    public static final String SECOND;

    static String observe() {
        return BoundaryEffects.trace + "|" + FIRST + "|" + SECOND;
    }

    static void patchAnchor() {
        BoundaryEffects.independent();
    }

    static {
        FIRST = BoundaryEffects.choose() ? BoundaryEffects.next("A") : BoundaryEffects.next("Z");
        SECOND = BoundaryEffects.next("B");
    }
}

public interface BoundaryProbe {
    String FIRST = BoundaryEffects.next("A");
    String SECOND = BoundaryEffects.next("B");
    static String observe() { return BoundaryEffects.trace + "|" + FIRST + "|" + SECOND; }
    static void patchAnchor() { BoundaryEffects.independent(); }
}

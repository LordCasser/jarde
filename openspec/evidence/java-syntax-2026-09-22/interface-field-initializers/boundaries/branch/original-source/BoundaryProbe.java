public interface BoundaryProbe {
    String FIRST = BoundaryEffects.choose() ? BoundaryEffects.next("A") : BoundaryEffects.next("Z");
    String SECOND = BoundaryEffects.next("B");
    static String observe() { return BoundaryEffects.trace + "|" + FIRST + "|" + SECOND; }
    static void patchAnchor() { BoundaryEffects.independent(); }
}

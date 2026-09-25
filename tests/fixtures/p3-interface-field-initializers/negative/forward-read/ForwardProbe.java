public interface ForwardProbe {
    int EARLY = ForwardProbe.LATE;
    int LATE = BoundaryEffects.value();

    static String observe() {
        return BoundaryEffects.trace + "|" + EARLY + "|" + LATE;
    }
}

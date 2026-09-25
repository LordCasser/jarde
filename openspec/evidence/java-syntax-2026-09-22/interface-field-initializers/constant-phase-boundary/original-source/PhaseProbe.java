public interface PhaseProbe {
    int EARLY = PhaseProbe.LATE;
    int LATE = BoundaryEffects.value();
    static String observe() { return EARLY + "|" + LATE; }
}

package defpackage;

/* JADX INFO: loaded from: PhaseProbe.class */
public interface PhaseProbe {
    public static final int LATE = 9;
    public static final int EARLY = LATE;

    static String observe() {
        return EARLY + "|" + LATE;
    }
}

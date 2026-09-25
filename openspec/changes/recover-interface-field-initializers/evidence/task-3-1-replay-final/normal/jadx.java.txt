package defpackage;

/* JADX INFO: loaded from: InterfaceInitProbe.class */
public interface InterfaceInitProbe {
    public static final int CONSTANT = 7;
    public static final String FIRST = InitEffects.next("A");
    public static final String SECOND = InitEffects.next("B");
    public static final int TOTAL = InitEffects.total(FIRST, SECOND);

    static String observe() {
        return InitEffects.trace + "|" + FIRST + "|" + SECOND + "|" + TOTAL + "|7";
    }
}

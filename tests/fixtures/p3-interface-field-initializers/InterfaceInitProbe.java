public interface InterfaceInitProbe {
    int CONSTANT = 7;
    String FIRST = InitEffects.next("A");
    String SECOND = InitEffects.next("B");
    int TOTAL = InitEffects.total(FIRST, SECOND);

    static String observe() {
        return InitEffects.trace + "|" + FIRST + "|" + SECOND + "|" + TOTAL + "|" + CONSTANT;
    }
}

final class ReferenceAliasBoundary {
    static volatile Object observed;

    static void consume(Object first, Object second) {
        observed = first;
        observed = second;
    }

    static void aliasSurvivesLaterStore() {
        Object value = Integer.valueOf(1);
        consume(value, value = "new".toString());
    }
}

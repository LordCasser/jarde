public final class ConstructorEffects {
    private static int calls;
    private static String events = "";

    private ConstructorEffects() {
    }

    public static void record(int value) {
        calls++;
        if (!events.isEmpty()) {
            events += ",";
        }
        events += value;
    }

    public static int calls() {
        return calls;
    }

    public static String events() {
        return events;
    }
}

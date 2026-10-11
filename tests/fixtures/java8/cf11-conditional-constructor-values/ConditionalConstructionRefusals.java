package cf11;

public class ConditionalConstructionRefusals {
    public static int events;
    public static int conditionCalls;
    public static int argumentCalls;
    public static int effectCalls;
    public static String sharedSlot;

    public static void reset() {
        events = 0;
        conditionCalls = 0;
        argumentCalls = 0;
        effectCalls = 0;
        sharedSlot = null;
    }

    private static boolean condition(boolean selected, boolean fail) {
        events = events * 10 + 1;
        conditionCalls++;
        if (fail) {
            throw new IllegalStateException("condition");
        }
        return selected;
    }

    private static String argument(String value, boolean fail) {
        events = events * 10 + 2;
        argumentCalls++;
        if (fail) {
            throw new IllegalArgumentException("argument");
        }
        return value;
    }

    private static void independentEffect() {
        events = events * 10 + 3;
        effectCalls++;
    }

    public static String localBoundary(boolean selected, String freshValue, String original) {
        String local = condition(selected, false)
                ? new String(argument(freshValue, false))
                : original;
        return local;
    }

    public static String sharedFieldAndReturn(boolean selected, String freshValue, String original) {
        return sharedSlot = condition(selected, false)
                ? new String(argument(freshValue, false))
                : original;
    }

    public static String nestedConditionals(boolean outerSelected, boolean innerSelected,
            String innerFresh, String innerOriginal, String outerFresh) {
        return condition(outerSelected, false)
                ? (condition(innerSelected, false)
                        ? new String(argument(innerFresh, false))
                        : innerOriginal)
                : new String(argument(outerFresh, false));
    }

    public static String independentBranchEffect(boolean selected, String freshValue, String original) {
        String result;
        if (condition(selected, false)) {
            independentEffect();
            result = new String(argument(freshValue, false));
        } else {
            result = original;
        }
        return result;
    }

    public static String handlerAroundConditional(boolean selected, boolean argumentFails,
            String freshValue, String original) {
        try {
            return condition(selected, false)
                    ? new String(argument(freshValue, argumentFails))
                    : original;
        } catch (IllegalArgumentException error) {
            events = events * 10 + 4;
            return original;
        }
    }
}

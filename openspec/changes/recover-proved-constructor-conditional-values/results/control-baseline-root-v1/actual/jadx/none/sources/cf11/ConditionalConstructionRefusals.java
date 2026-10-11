package cf11;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/ConditionalConstructionRefusals.class */
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
        events = (events * 10) + 1;
        conditionCalls++;
        if (fail) {
            throw new IllegalStateException("condition");
        }
        return selected;
    }

    private static String argument(String value, boolean fail) {
        events = (events * 10) + 2;
        argumentCalls++;
        if (fail) {
            throw new IllegalArgumentException("argument");
        }
        return value;
    }

    private static void independentEffect() {
        events = (events * 10) + 3;
        effectCalls++;
    }

    public static String localBoundary(boolean selected, String freshValue, String original) {
        String str;
        if (condition(selected, false)) {
            str = new String(argument(freshValue, false));
        } else {
            str = original;
        }
        String local = str;
        return local;
    }

    public static String sharedFieldAndReturn(boolean selected, String freshValue, String original) {
        String str;
        if (condition(selected, false)) {
            str = new String(argument(freshValue, false));
        } else {
            str = original;
        }
        sharedSlot = str;
        return str;
    }

    public static String nestedConditionals(boolean outerSelected, boolean innerSelected, String innerFresh, String innerOriginal, String outerFresh) {
        if (condition(outerSelected, false)) {
            if (condition(innerSelected, false)) {
                return new String(argument(innerFresh, false));
            }
            return innerOriginal;
        }
        return new String(argument(outerFresh, false));
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

    public static String handlerAroundConditional(boolean selected, boolean argumentFails, String freshValue, String original) {
        try {
            if (condition(selected, false)) {
                return new String(argument(freshValue, argumentFails));
            }
            return original;
        } catch (IllegalArgumentException e) {
            events = (events * 10) + 4;
            return original;
        }
    }
}

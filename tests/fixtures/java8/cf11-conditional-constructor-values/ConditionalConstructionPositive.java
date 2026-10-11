package cf11;

public class ConditionalConstructionPositive {
    public static int events;
    public static int conditionCalls;
    public static int argumentCalls;
    public static String staticSlot;
    public String instanceSlot;

    public static void reset() {
        events = 0;
        conditionCalls = 0;
        argumentCalls = 0;
        staticSlot = null;
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

    public static String oneConstructedArm(boolean selected, boolean conditionFails,
            boolean argumentFails, String original) {
        return condition(selected, conditionFails)
                ? new String(argument(original, argumentFails))
                : original;
    }

    public static String twoConstructedArms(boolean firstSelected, boolean conditionFails,
            boolean firstArgumentFails, boolean secondArgumentFails,
            String firstValue, String secondValue) {
        return condition(firstSelected, conditionFails)
                ? new String(argument(firstValue, firstArgumentFails))
                : new String(argument(secondValue, secondArgumentFails));
    }

    public static void writeStaticSlot(boolean selected, boolean conditionFails,
            boolean argumentFails, String original) {
        staticSlot = condition(selected, conditionFails)
                ? new String(argument(original, argumentFails))
                : original;
    }

    public void writeInstanceSlot(boolean selected, boolean conditionFails,
            boolean argumentFails, String original) {
        this.instanceSlot = condition(selected, conditionFails)
                ? new String(argument(original, argumentFails))
                : original;
    }
}

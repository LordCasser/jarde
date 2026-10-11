package cf11;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/ConditionalConstructionPositive.class */
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

    public static String oneConstructedArm(boolean selected, boolean conditionFails, boolean argumentFails, String original) {
        if (condition(selected, conditionFails)) {
            return new String(argument(original, argumentFails));
        }
        return original;
    }

    public static String twoConstructedArms(boolean firstSelected, boolean conditionFails, boolean firstArgumentFails, boolean secondArgumentFails, String firstValue, String secondValue) {
        if (condition(firstSelected, conditionFails)) {
            return new String(argument(firstValue, firstArgumentFails));
        }
        return new String(argument(secondValue, secondArgumentFails));
    }

    public static void writeStaticSlot(boolean selected, boolean conditionFails, boolean argumentFails, String original) {
        String str;
        if (condition(selected, conditionFails)) {
            str = new String(argument(original, argumentFails));
        } else {
            str = original;
        }
        staticSlot = str;
    }

    public void writeInstanceSlot(boolean selected, boolean conditionFails, boolean argumentFails, String original) {
        String str;
        if (condition(selected, conditionFails)) {
            str = new String(argument(original, argumentFails));
        } else {
            str = original;
        }
        this.instanceSlot = str;
    }
}

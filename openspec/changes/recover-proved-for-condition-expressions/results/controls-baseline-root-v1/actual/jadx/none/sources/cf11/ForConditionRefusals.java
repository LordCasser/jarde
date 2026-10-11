package cf11;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/ForConditionRefusals.class */
public class ForConditionRefusals {
    static int counter;
    static int limit;
    static int limitCalls;

    static void tick() {
        counter++;
    }

    static int limit() {
        limitCalls++;
        return limit;
    }

    static int sumWithLeadingTickAndBreak() {
        int i = 0;
        int sum = 0;
        while (true) {
            tick();
            if (i < limit()) {
                sum += i;
                i++;
            } else {
                return sum;
            }
        }
    }

    static int sumWithAssignedCallBound() {
        int i = 0;
        int sum = 0;
        while (true) {
            int i2 = i;
            int bound = limit();
            if (i2 < bound) {
                sum += i;
                i++;
            } else {
                return sum + bound;
            }
        }
    }
}

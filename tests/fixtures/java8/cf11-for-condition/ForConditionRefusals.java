package cf11;

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
            if (i >= limit()) {
                break;
            }
            sum += i;
            i++;
        }
        return sum;
    }

    static int sumWithAssignedCallBound() {
        int bound = 0;
        int i = 0;
        int sum = 0;
        while (i < (bound = limit())) {
            sum += i;
            i++;
        }
        return sum + bound;
    }
}

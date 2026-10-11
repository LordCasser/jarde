package cf11;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/ForConditionEffects.class */
public class ForConditionEffects {
    static int events;
    static int offsetCalls;
    static int limitCalls;
    static int offsetValue;
    static int limitValue;
    static int offsetThrowAt;
    static int limitThrowAt;
    static int dynamicLimit;

    static int offset() {
        offsetCalls++;
        events = (events * 10) + 1;
        if (offsetThrowAt != 0 && offsetCalls == offsetThrowAt) {
            throw new IllegalStateException("offset");
        }
        return offsetValue;
    }

    static int limit() {
        limitCalls++;
        events = (events * 10) + 2;
        if (limitThrowAt != 0 && limitCalls == limitThrowAt) {
            throw new IllegalStateException("limit");
        }
        return limitValue;
    }

    static int sumWithFunctionBounds() {
        int sum = 0;
        for (int i = 0; i < offset() + limit(); i++) {
            sum += i;
            events = (events * 10) + 3;
        }
        return sum;
    }

    static int sumWithFunctionBoundsAndContinue() {
        int sum = 0;
        for (int i = 0; i < offset() + limit(); i++) {
            if (i == 0) {
                events = (events * 10) + 4;
            } else {
                sum += i;
                events = (events * 10) + 3;
            }
        }
        return sum;
    }

    static int sumWithChangingFieldBound() {
        int sum = 0;
        for (int i = 0; i < dynamicLimit; i++) {
            sum += i;
            events = (events * 10) + 3;
            dynamicLimit--;
        }
        return sum;
    }
}

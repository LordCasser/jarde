package cf11;

public class ForConditionReads {
    static int fieldBound;
    static int negativeField;

    static int sumWithChangingFieldBound() {
        int sum = 0;
        for (int i = 0; i < fieldBound; i++) {
            sum += i;
            fieldBound--;
        }
        return sum;
    }

    static int sumWithArrayLength(int[] values) {
        int sum = 0;
        for (int i = 0; i < values.length; i++) {
            sum += i;
        }
        return sum;
    }

    static int countWithArrayElement(int[] values) {
        int count = 0;
        for (int i = 0; i < values[i]; i++) {
            count++;
        }
        return count;
    }

    static int sumWithNegatedField() {
        int sum = 0;
        for (int i = 0; i < -negativeField; i++) {
            sum += i;
        }
        return sum;
    }

    static int sumWithNegatedParameter(int value) {
        int sum = 0;
        for (int i = 0; i < -value; i++) {
            sum += i;
        }
        return sum;
    }

    static int oldWhileWithWrittenLocalBound() {
        int bound = 3;
        int i = 0;
        int count = 0;
        while (i < bound) {
            count++;
            bound--;
            i++;
        }
        return count;
    }
}

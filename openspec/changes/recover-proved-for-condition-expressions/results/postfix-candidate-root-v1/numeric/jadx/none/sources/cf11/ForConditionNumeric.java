package cf11;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/ForConditionNumeric.class */
public class ForConditionNumeric {
    static int countLong(long[] values, long bound) {
        int count = 0;
        for (int i = 0; values[i] < bound; i++) {
            count++;
        }
        return count;
    }

    static int countFloat(float[] values, float bound) {
        int count = 0;
        for (int i = 0; values[i] < bound; i++) {
            count++;
        }
        return count;
    }

    static int countDouble(double[] values, double bound) {
        int count = 0;
        for (int i = 0; values[i] < bound; i++) {
            count++;
        }
        return count;
    }
}

public class PlainOneArmLoops {
    public static int prefixWhile(boolean e, int n) {
        int sum = 0;
        if (e) {
            int j = 0;
            while (j < n) {
                sum += j;
                j++;
            }
        }
        return sum;
    }

    public static int noPrefix(boolean e, int n) {
        int v = 0;
        if (e) {
            while (v < n) {
                v++;
            }
        }
        return v;
    }

    public static int loopAndTail(boolean e, int n) {
        int sum = 0;
        if (e) {
            int j = 0;
            while (j < n) {
                sum += j;
                j++;
            }
            sum += 100;
        }
        return sum;
    }

    public static int takenArm(boolean e, int n) {
        int sum = 0;
        if (!e) {
            int j = 0;
            while (j < n) {
                sum += j;
                j++;
            }
        }
        return sum;
    }
}

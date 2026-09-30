public class Cf10Refused {
    static int shared(int[] data) {
        int sum = 0;
        int pick = 0;
        for (int i = 0; i < data.length; i += 2) {
            try {
                pick = sum = data[i];
            } catch (IllegalStateException e) {
                sum -= 1;
            }
        }
        return sum + pick;
    }

    static int crossBlock(int[] data, boolean flag) {
        int sum = 0;
        for (int i = 0; i < data.length; i += 2) {
            int t = flag ? data[i] : 0;
            try {
                sum += t;
            } catch (IllegalStateException e) {
                sum -= 1;
            }
        }
        return sum;
    }

    static int intervalEffect(int[] data) {
        int sum = 0;
        int k = 0;
        for (int i = 0; i < data.length; i += 2) {
            try {
                sum = sum + (k = 2) + data[i];
            } catch (IllegalStateException e) {
                sum -= 1;
            }
        }
        return sum + k;
    }

    public static void main(String[] args) {
        int[] d = {1, 2, 3, 4, 5};
        System.out.println(shared(d));
        System.out.println(crossBlock(d, true));
        System.out.println(intervalEffect(d));
    }
}

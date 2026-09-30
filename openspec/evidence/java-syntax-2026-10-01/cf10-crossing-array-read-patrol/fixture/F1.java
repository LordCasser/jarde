public class F1 {
    public static int stepTwo(int[] data) {
        int sum = 0;
        for (int i = 0; i < data.length; i += 2) {
            sum += data[i];
        }
        return sum;
    }
    public static String stepTwoWithCall(int[] data) {
        StringBuilder b = new StringBuilder();
        for (int i = 0; i < data.length; i += 2) {
            b.append(pick(data[i]));
        }
        return b.toString();
    }
    static int pick(int v) { return v * 3; }
    public static void main(String[] x) {
        int[] d = {1, 2, 3, 4, 5};
        System.out.println(stepTwo(d));
        System.out.println(stepTwoWithCall(d));
    }
}

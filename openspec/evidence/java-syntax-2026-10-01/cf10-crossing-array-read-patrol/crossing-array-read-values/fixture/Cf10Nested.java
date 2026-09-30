public class Cf10Nested {
    static int inLoop(int[] data) {
        int sum = 0;
        for (int i = 0; i < data.length; i += 2) {
            sum += data[i];
            try {
                try {
                    sum += risky(data[i]);
                } catch (IllegalStateException e) {
                    sum -= 1;
                }
            } catch (ArithmeticException e) {
                sum += data[i + 1];
            }
        }
        return sum;
    }

    static int aroundLoop(int[] data) {
        int sum = 0;
        try {
            for (int i = 0; i < data.length; i += 2) {
                sum += data[i];
                try {
                    sum += risky(data[i]);
                } catch (IllegalStateException e) {
                    sum -= 1;
                }
            }
        } catch (ArithmeticException e) {
            sum += data[0];
        }
        return sum;
    }

    static int risky(int v) {
        if (v == 3) {
            throw new IllegalStateException("r");
        }
        return v;
    }

    public static void main(String[] args) {
        int[] d = {1, 2, 3, 4, 5};
        System.out.println(inLoop(d));
        System.out.println(aroundLoop(d));
    }
}

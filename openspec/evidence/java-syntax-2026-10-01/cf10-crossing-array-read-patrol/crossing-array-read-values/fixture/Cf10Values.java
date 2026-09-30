public class Cf10Values {
    public static int intSum(int[] data) {
        int sum = 0;
        for (int i = 0; i < data.length; i += 2) {
            sum += data[i];
            try {
                sum += risky(data[i]);
            } catch (IllegalStateException e) {
                sum -= 1;
            }
        }
        return sum;
    }

    public static double doubleSum(double[] data) {
        double sum = 0;
        for (int i = 0; i < data.length; i += 2) {
            sum += data[i];
            try {
                noise(data[i]);
            } catch (IllegalStateException e) {
                sum = data[1];
            }
        }
        return sum;
    }

    public static String lastRef(String[] parts) {
        String cur = parts[0];
        for (int i = 1; i < parts.length; i += 2) {
            try {
                cur = parts[i];
                noise(cur);
            } catch (IllegalStateException e) {
                cur = parts[0];
            }
        }
        return cur;
    }

    public static int catchMultiWrite(int[] data) {
        int sum = 0;
        for (int i = 0; i < data.length; i += 2) {
            sum += data[i];
            try {
                sum += risky(data[i]);
            } catch (IllegalStateException e) {
                sum = data[1];
                sum += data[2];
            }
        }
        return sum;
    }

    public static int twoTries(int[] data) {
        int sum = 0;
        for (int i = 0; i < data.length; i += 2) {
            sum += data[i];
            try {
                sum += risky(data[i]);
            } catch (IllegalStateException e) {
                sum -= 1;
            }
            try {
                noise2(sum);
            } catch (ArithmeticException e) {
                sum = data[1];
            }
        }
        return sum;
    }

    static int risky(int v) {
        if (v == 3) {
            throw new IllegalStateException("r");
        }
        return v;
    }

    static void noise2(int v) {
        if (v == 4) {
            throw new ArithmeticException("m");
        }
    }

    static void noise(double v) {
        if (v == 3.0) {
            throw new IllegalStateException("n");
        }
    }

    static void noise(String v) {
        if ("boom".equals(v)) {
            throw new IllegalStateException("n");
        }
    }

    public static void main(String[] args) {
        int[] d = {1, 2, 3, 4, 5};
        System.out.println(intSum(d));
        double[] f = {1.5, 2.5, 3.0, 4.0, 5.5};
        System.out.println(doubleSum(f));
        String[] p = {"a", "boom", "c", "d", "e", "f"};
        System.out.println(lastRef(p));
        System.out.println(catchMultiWrite(d));
        System.out.println(twoTries(d));
    }
}

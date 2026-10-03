public class A1 {
    public static int check(int x) {
        assert x > 0 : "positive: " + x;
        return x * 2;
    }
    public static int plainAssert(int x) {
        assert x != 0;
        return 100 / x;
    }
    static class Sub {
        int m(int v) {
            assert v > 1 : v;
            return v - 1;
        }
    }
    public static void main(String[] a) {
        System.out.println(check(5));
        System.out.println(plainAssert(4));
        System.out.println(new Sub().m(3));
        System.out.println(check(-1));
    }
}

public class IBRMulti {
    static int a;
    static {
        a = 1;
    }
    static int b = 2;
    static {
        if (Boolean.getBoolean("ibr-multi-boom")) { throw new IllegalStateException("multi-fail"); }
        a = a + b;
    }
    static int c = a * 3;
    static {
        b = a + c;
    }
    public static void main(String[] args) {
        System.out.println(a + ":" + b + ":" + c);
    }
}

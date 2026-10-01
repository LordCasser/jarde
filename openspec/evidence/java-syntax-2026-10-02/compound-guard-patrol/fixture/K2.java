public class K2 {
    static int a = Other.first;
    static int b;
    static {
        b = Other.second();
    }
    static int sum() { return a + b; }
    public static void main(String[] x) {
        System.out.println(sum());
    }
}
class Other {
    static int first = 5;
    static int second() {
        int total = 0;
        for (int i = 0; i < first; i++) { total += i; }
        return total;
    }
}

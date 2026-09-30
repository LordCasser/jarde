public class C3 {
    public static int work(int x) { return x + 1; }
    public static String five() {
        int sum = 0;
        try { sum += work(1); } catch (NoSuchFieldError unused) { }
        try { sum += work(2); } catch (IllegalStateException unused) { }
        try { sum += work(3); } catch (NoSuchFieldError unused) { }
        return "s" + sum;
    }
    public static String popstmt() {
        StringBuilder b = new StringBuilder();
        b.append('a');
        b.append('b');
        return b.toString();
    }
    public static void main(String[] args) {
        System.out.println(five());
        System.out.println(popstmt());
    }
}

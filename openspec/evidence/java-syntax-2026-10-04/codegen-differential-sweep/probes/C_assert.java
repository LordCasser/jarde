public class C_assert {
    static int f(int x){ assert x > 0 : "neg"; return x; }
    public static void main(String[] a){ System.out.println(f(5)); }
}

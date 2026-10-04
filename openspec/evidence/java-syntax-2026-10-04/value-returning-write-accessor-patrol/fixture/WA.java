public class WA {
    private int v = 1;
    class Setter { void set(int x) { v = x; } }     // 内部类写外部私有字段 → 写访问器
    public static void main(String[] a){ WA o=new WA(); o.new Setter().set(5); System.out.println(o.v); }
}

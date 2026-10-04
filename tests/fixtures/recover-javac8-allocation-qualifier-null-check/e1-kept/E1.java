public class E1 {
    class In {
        int v() { return 4; }
    }
    public static void main(String[] args) {
        Class c = null;
        int r = new E1().new In().v();
        System.out.println(r + (c == null ? 0 : 1));
    }
}

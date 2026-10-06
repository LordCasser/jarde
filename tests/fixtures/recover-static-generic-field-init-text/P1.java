public class P1 {
    static class Box { int v; Box(int v){ this.v = v; } }
    static Box b = new Box(1);
    public static void main(String[] a){ System.out.println(b.v); }
}

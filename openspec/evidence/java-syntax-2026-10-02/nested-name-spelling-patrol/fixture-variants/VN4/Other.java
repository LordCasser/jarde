public class Other {
    public static class Inner {
        public static int twice(int x) { return x * 2; }
        public int id(int x) { return x + 100; }
    }
    public static class Box { public int size(int x) { return x * x; } }
    public interface Mark { String tag(); }
    public static class Tagged extends Box implements Mark {
        public String tag() { return "t"; }
    }
}

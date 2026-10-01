public class X3 {
    static class B {
        final String s;
        B(String s) { this.s = s; }
        public String toString() { return "B(" + s + ")"; }
    }
    static class C {
        final String s;
        C(String s) { this.s = s; }
        public String toString() { return "C(" + s + ")"; }
    }
    static class TwoNested {
        final B b;
        final C c;
        TwoNested(B b, C c) { this.b = b; this.c = c; }
        public String toString() { return "TwoNested(" + b + "," + c + ")"; }
    }
    static class TwoSame {
        final B x;
        final B y;
        TwoSame(B x, B y) { this.x = x; this.y = y; }
        public String toString() { return "TwoSame(" + x + "," + y + ")"; }
    }
    static class Tagged {
        final String tag;
        final B b;
        Tagged(String tag, B b) { this.tag = tag; this.b = b; }
        public String toString() { return "Tagged(" + tag + "," + b + ")"; }
    }
    public static String doubleNested() { return String.valueOf(new TwoNested(new B("y"), new C("z"))); }
    public static String secondPosition() { return String.valueOf(new Tagged("first", new B("second"))); }
    public static String sameClassTwice() { return String.valueOf(new TwoSame(new B("1"), new B("2"))); }
    public static void main(String[] args) {
        System.out.println(doubleNested());
        System.out.println(secondPosition());
        System.out.println(sameClassTwice());
    }
}

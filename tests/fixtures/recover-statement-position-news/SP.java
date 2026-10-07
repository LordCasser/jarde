// `recover-statement-position-news`'s positives: every statement-position `new` whose arguments
// are a constant, a direct local/parameter read or a proved nested construction, beside the
// consumed-position shapes that must not move.
public class SP {
    int n;

    SP() { System.out.println("sp0"); }

    SP(int v) { n = v; System.out.println("sp" + v); }

    SP(SP other) { n = other.n; System.out.println("sp#" + n); }

    static class Inner {
        Inner() { System.out.println("inner"); }
    }

    static void argless() { new SP(); }

    static void withArg() { new SP(7); }

    static void fromArg(int x) { new SP(x); }

    static void nestedArgument() { new SP(new SP(1)); }

    static void consumed() { int r = new SP(3).n; }

    static void mixed() {
        new SP(4);
        int r = new SP(5).n;
        new SP(6);
        System.out.println(new SP(8).n);
        new Inner();
    }

    public static void main(String[] args) {
        argless();
        withArg();
        fromArg(2);
        nestedArgument();
        consumed();
        mixed();
    }
}

public class NestedEval {
    static int calls;
    static int tick(int x) { calls++; return x; }
    public static int nestedLocal(int x) { return (x + 1) + ++x; }
    public static int nestedPlain(int x) { return (x + 1) + (x + 2); }
    public static int nestedCall(int x) { return tick(x) + ++x; }
}

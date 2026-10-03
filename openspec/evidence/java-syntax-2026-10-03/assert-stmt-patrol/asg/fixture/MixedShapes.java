public class V1 {
    static boolean flag;
    static int counter = 7;
    static void both(boolean a, boolean b) {
        assert a && b : "conj";
        counter++;
    }
    static void disj(boolean a, boolean b) {
        assert a || b;
    }
    static void local(boolean f) {
        assert f;
    }
    static void negated(boolean f) {
        assert !f : counter;
    }
    static void call(int v) {
        assert check(v) : "call:" + v;
    }
    static boolean check(int v) { return v > 0; }
    static void two(int x) {
        assert x > 0 : "first";
        assert x < 100 : "second";
    }
}

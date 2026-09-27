package em22;

public class Arithmetic {
    private int calls;

    public int multiply(int a) {
        return (a + 2) * 3;
    }

    public int sum(int a, int b, int c) {
        return a + b + c;
    }

    public int subtract(int a, int b, int c) {
        return a - (b - c);
    }

    public int divide(int a, int b, int c) {
        return a / (b / c);
    }

    public boolean or(boolean a, boolean b, boolean c) {
        return a | b | c;
    }

    public boolean and(boolean a, boolean b, boolean c) {
        return a & b & c;
    }

    public int notInt(int a) {
        return ~a;
    }

    public long notLong(long a) {
        return ~a;
    }

    public boolean flip(boolean a) {
        return a ^ true;
    }

    public boolean flipCall() {
        return left() ^ true;
    }

    public boolean sameCall() {
        return left() ^ false;
    }

    private boolean left() {
        calls++;
        return true;
    }

    private boolean right() {
        calls++;
        return false;
    }

    public boolean eagerOr() {
        return left() | right();
    }

    public int calls() {
        return calls;
    }
}

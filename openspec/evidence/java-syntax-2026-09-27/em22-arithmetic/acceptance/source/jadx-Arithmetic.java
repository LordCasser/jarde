package em22;

/* JADX INFO: loaded from: input.jar:em22/Arithmetic.class */
public class Arithmetic {
    private int calls;

    public int multiply(int i) {
        return (i + 2) * 3;
    }

    public int sum(int i, int i2, int i3) {
        return i + i2 + i3;
    }

    public int subtract(int i, int i2, int i3) {
        return i - (i2 - i3);
    }

    public int divide(int i, int i2, int i3) {
        return i / (i2 / i3);
    }

    public boolean or(boolean z, boolean z2, boolean z3) {
        return z | z2 | z3;
    }

    public boolean and(boolean z, boolean z2, boolean z3) {
        return z & z2 & z3;
    }

    public int notInt(int i) {
        return i ^ (-1);
    }

    public long notLong(long j) {
        return j ^ (-1);
    }

    public boolean flip(boolean z) {
        return !z;
    }

    public boolean flipCall() {
        return !left();
    }

    public boolean sameCall() {
        return left();
    }

    private boolean left() {
        this.calls++;
        return true;
    }

    private boolean right() {
        this.calls++;
        return false;
    }

    public boolean eagerOr() {
        return left() | right();
    }

    public int calls() {
        return this.calls;
    }
}

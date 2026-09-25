// The one-armed-branch fixture: a branch whose taken successor *is* the immediate post-dominator of
// the test block. `javac --release 8 -g:none` puts the arm on the fall-through side (`ifle` jumps
// straight to the join), so the shape is an `if` with no `else` — not two arms, and not a refusal.
public class OneArmed {
    // `iconst_1; istore_1; iload_0; ifle <join>; iload_0; istore_1; iload_1; ireturn`.
    public static int oneArmed(int n) {
        int x = 1;
        if (n > 0) {
            x = n;
        }
        return x;
    }

    // The opposite arm: the *fall-through* successor is the join and the assignment is the one the
    // branch transfers to. The arm is not empty, so the `else` is written.
    public static int elseOnly(int n) {
        int x = 1;
        if (n > 0) {
        } else {
            x = n;
        }
        return x;
    }

    // The control: two real arms that meet at the join, unchanged by the one-armed recovery.
    public static int bothArms(int n) {
        int x;
        if (n > 0) {
            x = 2;
        } else {
            x = 3;
        }
        return x;
    }
}

// P3 fixture: one local's type is decided once, before its statements are built, so the hoisted
// declaration path and the in-place declaration path cannot disagree.
//
// Every member is a shape the review measured on the real class file: `copied`, `swapped` and
// `relayed` are the defect (a local filled from a local the body declared `boolean`), `literalArmed`
// and `intLocal` are the boundaries a literal must not move, `fromParameter` is the descriptor-only
// control, and `unproven`/`conflicted` are the two refusals the decision's evidence rule produces.
public class HoistedBoolean {

    // The review's sample: `c`'s writes are in two arms, its use is after the join, so its
    // declaration is hoisted above the branch — where the value it stores is a read of the local
    // `a` the body itself declared `boolean` (one arm) and a read of the `boolean` parameter
    // (the other).
    public static int copied(boolean b, int n) {
        boolean a = b;
        boolean c;
        if (n == 0) {
            c = a;
        } else {
            c = b;
        }
        if (c) {
            return 1;
        }
        return 0;
    }

    // The same shape with the arms swapped: the write the bytecode reaches first is the
    // descriptor-proven one instead of the copied local. The conclusion may not depend on it.
    public static int swapped(boolean b, int n) {
        boolean a = b;
        boolean c;
        if (n == 0) {
            c = b;
        } else {
            c = a;
        }
        if (c) {
            return 1;
        }
        return 0;
    }

    // A copy chain through two more locals: `y`'s evidence arrives from `x` and `z`'s from `y`, so
    // one pass over the writes in bytecode order is not enough — the decision has to reach a
    // fixpoint before the first statement is built.
    public static boolean relayed(boolean b) {
        boolean x = b;
        boolean y;
        boolean z;
        if (b) {
            y = x;
            z = y;
        } else {
            y = x;
            z = y;
        }
        return z;
    }

    // The literal-armed boundary, recorded and not closed: the only values written are `true` and
    // `false`, and a `0`/`1` literal is how both a `boolean` and an `int` are pushed, so the frames'
    // `int` decides and the text stays an `int` local — self-consistent, compilable and equal to the
    // original's answers.
    public static int literalArmed(boolean b) {
        boolean x;
        if (b) {
            x = true;
        } else {
            x = false;
        }
        if (x) {
            return 1;
        }
        return 0;
    }

    // The descriptor-only control (the predecessor change's `pick` shape): every write stores a
    // read of a `Z` parameter, so the hoisted declaration is `boolean` before and after.
    public static boolean fromParameter(boolean b, int n) {
        boolean c;
        if (n == 0) {
            c = b;
        } else {
            c = b;
        }
        return c;
    }

    // The pure-`int` control: `0`/`1` is the value, the frame's `int` is the type, and nothing here
    // is a boolean.
    public static int intLocal(int n) {
        int x = 0;
        if (n == 0) {
            x = 1;
        } else {
            x = n;
        }
        return x;
    }

    // The boundary of `literalArmed`, one position along: the same body in a `Z` method, where the
    // position requires a boolean the evidence does not have. Refused, as before this change.
    public static boolean unproven(boolean b) {
        boolean x;
        if (b) {
            x = true;
        } else {
            x = false;
        }
        return x;
    }

    // A write that cannot be spelled as the type the variable's own first write decided: the first
    // write is a literal (the frame's `int`), the second is a descriptor-proven `boolean`. The
    // assignment is a conflict, so the structure it belongs to is refused instead of published.
    public static int conflicted(boolean b, int n) {
        boolean c;
        if (n == 0) {
            c = true;
        } else {
            c = b;
        }
        if (c) {
            return 1;
        }
        return 0;
    }
}

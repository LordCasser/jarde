/**
 * The P3 3.1 shapes: where a local's declaration has to be written, and what a local's name is a
 * name for.
 *
 * <p>The members are compiled twice by the fixture's README — once with {@code -g:none} (the
 * no-debug sample every slot name is derived for) and once with {@code -g} (a body whose
 * {@code LocalVariableTable} states the source names) — so that the same shapes are read both with
 * and without debug evidence.
 *
 * <p>{@link #scope(boolean)} is the review's P3-R3 counterexample: the slot is written in the
 * {@code then} arm and in the {@code else} arm and read after the join, so a declaration written at
 * the first write is out of scope in two places.
 *
 * <p>{@link #simple()} is the control in the other direction: one write and one read in one
 * straight run, where a declaration at the write is in scope everywhere and must keep its initial
 * value. {@link #armOnly(int)} holds both shapes at once: one slot used across the arms and after
 * the join, one slot declared and used inside the arm alone — the second must stay inside the arm.
 *
 * <p>{@link #reuse(boolean, int)} is the slot-reuse shape: two variables in disjoint scopes take
 * one slot. {@link #after(long, int)} and {@link #reassign(long, int)} are the category-2 shapes:
 * the {@code long} occupies two slots, so the {@code int} parameter sits at slot 2 and a miscount
 * would declare a parameter as a local. {@link #receiver(long)} is the same with a receiver below
 * it.
 */
public final class Scope {

    /** P3-R3: written in both arms, read after the join. */
    public static int scope(boolean b) {
        int x;
        if (b) {
            x = 1;
        } else {
            x = 2;
        }
        return x;
    }

    /** The control: one write and one read, both in the same straight run. */
    public static int simple() {
        int x = 5;
        return x;
    }

    /** One slot used across the arms and after the join, one used inside the arm alone. */
    public static int armOnly(int n) {
        int y = 0;
        if (n > 0) {
            int z = n + 1;
            y = z;
        } else {
            y = 1;
        }
        return y;
    }

    /** Two variables in disjoint scopes, sharing the slot the compiler reuses for them. */
    public static int reuse(boolean b, int seed) {
        int a;
        if (b) {
            int c = seed + 1;
            a = c;
        } else {
            int d = seed + 2;
            a = d;
        }
        return a;
    }

    /** A category-2 parameter: the {@code int} parameter sits at slot 2, not at slot 1. */
    public static int after(long a, int b) {
        return b;
    }

    /** The same layout with a write: a miscount would declare the parameter as a local. */
    public static int reassign(long a, int b) {
        b = b + 1;
        return b;
    }

    /** A receiver below a category-2 parameter: {@code this} is slot 0, the {@code long} 1 and 2. */
    public long receiver(long a) {
        return a;
    }
}

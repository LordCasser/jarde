// P3 2c.25: a switch *expression* is an ordinary `lookupswitch` whose arms leave one value each on
// one shared `ireturn` — the default arm falls into it, and the `yield x` of a block-shaped arm
// leaves the load of the local it stored.
//
//     javac --release 21 -g:none -d v21 Joined.java
//
// `expr` is the constant-armed shape: each arm pushes one constant and jumps to the join, whose
// single instruction is the member's own `ireturn`. `yielded` is the same join with one arm that
// stores a local first: its `return local1` has to keep the store the arm already wrote.
public class Joined {
    static int expr(int n) {
        return switch (n) {
            case 1 -> 2;
            case 2 -> 3;
            default -> 0;
        };
    }

    static int yielded(int n) {
        return switch (n) {
            case 1 -> { int x = n + 1; yield x; }
            default -> 0;
        };
    }
}

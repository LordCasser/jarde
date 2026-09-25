// The forward-join fixture: two branches whose *shared* successor is a join the immediate
// post-dominator does not state, because one of the arms leaves the method before reaching it.
// `javac --release 8 -g:none` writes both `&&` and `||` as two forward branches onto one block, so
// each member is an `if` whose join is stated by convergence and not by post-domination.
public class Join {
    // `iload_0; ifle <join>; iload_1; ifle <join>; iconst_1; ireturn; <join>: iconst_0; ireturn`:
    // block `<join>` is reached by two forward edges, and the `return 1` path never reaches it — so
    // the join is not the branch's post-dominator, and the outer `if` is one-armed with the inner
    // `if` as its body.
    public static int both(int a, int b) {
        if (a > 0 && b > 0) {
            return 1;
        }
        return 0;
    }

    // `iload_0; ifgt <join>; iload_1; ifle <exit>; iconst_1; ireturn; <join>: iconst_0; ireturn`:
    // the same kind of forward join, with the *taken* edge of the outer branch arriving at it.
    public static int either(int a, int b) {
        if (a > 0 || b > 0) {
            return 1;
        }
        return 0;
    }

    // The control: a loop is still a loop. Its exit is reached by the header's own branch and by
    // nothing else, so no predecessor other than the branch node converges on it, and the header is
    // read as a loop before any branch arm is examined.
    public static int again(int n) {
        while (n > 0) {
            n = n - 1;
        }
        return n;
    }
}

/// The branching guard body's own anchor (`recover-branching-guard-body`).
///
/// Both members are the nested-lock statement whose **protected body branches** — the shape the
/// nested-lock slice registered as its third measured boundary (`MLProbe.nestedLocksBranching`,
/// refused at BCI 55) and this slice presents:
///
/// * the branch puts the release copy in a block of its own, because the protected range's end is
///   that block's own start;
/// * that block carries no exception edge of its own (the range stops where it begins), so the
///   canonical graph fuses the method's trailing value-less `return` into it;
/// * the void completion's transfer therefore has **no successor block to state**, and the
///   certificate reads the completion as the fused block's tail instead.
///
/// `singleIf` is the measured anchor verbatim (the patrol's own shape: `if (fail) throw …;`), and
/// `ifElse` is the same guard with both arms written — the two variants the slice's task names.
/// The protected body itself is the walk's own region tree, so nothing about either arm is
/// special-cased: the existing `if` shapes present it.
public class BG {
    private final java.util.concurrent.locks.ReentrantLock a =
            new java.util.concurrent.locks.ReentrantLock();
    private final java.util.concurrent.locks.ReentrantLock b =
            new java.util.concurrent.locks.ReentrantLock();
    private int count = 0;

    void singleIf(boolean fail) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (fail) {
                throw new IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void ifElse(boolean take) {
        this.a.lock();
        this.b.lock();
        try {
            if (take) {
                this.count++;
            } else {
                this.count--;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }
}

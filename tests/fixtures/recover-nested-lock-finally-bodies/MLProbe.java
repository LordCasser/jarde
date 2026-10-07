/// The registered boundaries of the nested-lock family, each measured rather than assumed: the
/// shapes one link beyond this change's own admission, compiled from source so each answer is
/// evidence about the certificate rather than about damaged bytes.
///
/// * `nestedTry()` — a `try`/`finally` **inside** the guarded range (the second lock's own
///   statement): the table states two rows over two handlers, and this certificate's own row set
///   is one row over one handler, so the nested statement stays refused;
/// * `threeLocks()` — three acquisitions and three releases: the multi-statement finally body's
///   own bound is two statements, so the third lock's statement stays refused;
/// * `nestedLocksBranching()` — the same two-lock guard whose **body branches**: the branch puts
///   the release copy in a block of its own, and the canonical graph fuses the method's own
///   trailing `return` into that block (nothing else enters it), so the void completion's transfer
///   has no successor block to state. The shape is the certificate's own except for that fused
///   layout, and it stays refused: a boundary registered rather than assumed.
public class MLProbe {
    private final java.util.concurrent.locks.ReentrantLock a =
            new java.util.concurrent.locks.ReentrantLock();
    private final java.util.concurrent.locks.ReentrantLock b =
            new java.util.concurrent.locks.ReentrantLock();
    private final java.util.concurrent.locks.ReentrantLock c =
            new java.util.concurrent.locks.ReentrantLock();
    private int count = 0;

    void nestedTry() {
        this.a.lock();
        try {
            this.b.lock();
            try {
                this.count++;
            } finally {
                this.b.unlock();
            }
        } finally {
            this.a.unlock();
        }
    }

    void threeLocks() {
        this.a.lock();
        this.b.lock();
        this.c.lock();
        try {
            this.count++;
        } finally {
            this.c.unlock();
            this.b.unlock();
            this.a.unlock();
        }
    }

    void nestedLocksBranching(boolean fail) {
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
}

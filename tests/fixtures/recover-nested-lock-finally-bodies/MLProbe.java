/// The registered boundaries of the nested-lock family, each measured rather than assumed: the
/// shapes one link beyond this change's own admission, compiled from source so each answer is
/// evidence about the certificate rather than about damaged bytes.
///
/// * `nestedTry()` — a `try`/`finally` **inside** the guarded range (the second lock's own
///   statement): the table states two rows over two handlers, and this certificate's own row set
///   is one row over one handler, so the nested statement stays refused;
/// * `threeLocks()` — three acquisitions and three releases: the multi-statement finally body's
///   own bound is two statements, so the third lock's statement stays refused.
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
}

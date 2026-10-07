/// The order-and-exception leg of the nested-lock family: the two locks are this fixture's own
/// `Order` recorder, so the `finally`'s release order is observable in the driver's log.
///
/// `nestedLocks`/`nestedLocksThrowing` are the anchor's two-lock guard — the release sequence is
/// the acquisition sequence's reverse, and the body may leave through an exception, in which case
/// the `finally` still runs both releases in that order. `interruptibly` is the single-lock guard
/// whose acquisition call can throw: the row's own range begins after the call, so a throw from it
/// never enters the range and the release must not run.
public final class MLOrder {
    private final Order a = new Order("a");
    private final Order b = new Order("b");
    private int count = 0;
    private boolean fail = false;

    void nestedLocks() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void nestedLocksThrowing() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            this.check();
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    private void check() {
        if (this.fail) {
            throw new IllegalStateException("body failed");
        }
    }

    void interruptibly() throws InterruptedException {
        this.a.lockInterruptibly();
        try {
            this.count++;
        } finally {
            this.a.unlock();
        }
    }

    int count() {
        return this.count;
    }

    void arm(boolean armed) {
        this.fail = armed;
    }
}

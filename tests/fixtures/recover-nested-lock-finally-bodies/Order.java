import java.util.concurrent.TimeUnit;
import java.util.concurrent.locks.Condition;
import java.util.concurrent.locks.Lock;

/// A lock that records its own operations. The driver reads the log, so the `finally`'s release
/// order — and whether the release ran at all when the acquisition threw — is **measured**, not
/// assumed: a `finally` that released in acquisition order, released a lock it never took, or ran
/// its release after a failed `lockInterruptibly` would leave a different log.
final class Order implements Lock {
    /// The operations every `Order` instance recorded, in the order they ran.
    static final StringBuilder LOG = new StringBuilder();

    /// The next `lockInterruptibly` throws instead of acquiring, so the driver can exercise the
    /// one path the bytecode's own row states: a throw from the acquisition never enters the
    /// protected range, so the release must not run.
    static boolean interruptNext;

    private final String name;
    private boolean held;

    Order(String name) {
        this.name = name;
    }

    public void lock() {
        LOG.append(this.name).append(".lock ");
        this.held = true;
    }

    public void lockInterruptibly() throws InterruptedException {
        if (interruptNext) {
            throw new InterruptedException(this.name + " interrupted");
        }
        LOG.append(this.name).append(".lockInterruptibly ");
        this.held = true;
    }

    public boolean tryLock() {
        LOG.append(this.name).append(".tryLock ");
        this.held = true;
        return true;
    }

    public boolean tryLock(long time, TimeUnit unit) {
        return this.tryLock();
    }

    public void unlock() {
        if (!this.held) {
            throw new IllegalStateException(this.name + " not held");
        }
        LOG.append(this.name).append(".unlock ");
        this.held = false;
    }

    public Condition newCondition() {
        throw new UnsupportedOperationException("no conditions on this lock");
    }

    static String log() {
        return LOG.toString();
    }

    static void reset() {
        LOG.setLength(0);
        interruptNext = false;
    }
}

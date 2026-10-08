import java.util.concurrent.TimeUnit;
import java.util.concurrent.locks.Condition;
import java.util.concurrent.locks.Lock;

/// A lock that records its own operations, so the driver reads the `finally`'s release **order**
/// and whether the release ran at all — measured, not assumed. A `finally` that released in the
/// acquisition order, released a lock it never took, or ran its release after a body that threw
/// would leave a different log.
///
/// This is the nested-lock fixture's own recorder, repeated here so this fixture stands on its own
/// bytes: the driver's expected lines are the same ones that fixture records.
final class Order implements Lock {
    /// The operations every `Order` instance recorded, in the order they ran.
    static final StringBuilder LOG = new StringBuilder();

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
    }
}

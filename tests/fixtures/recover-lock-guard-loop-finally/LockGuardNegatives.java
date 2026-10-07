/// The shapes beside the lock guard: each one breaks exactly one link of the certificate's own
/// proof, and each is a **verifier-valid** Java 8 class compiled from this source, so a refusal is
/// evidence about the proof rather than about hand-damaged bytes.
///
/// * `lockMismatch()` — the one invocation before the protected range takes `lock`, and both
///   cleanup copies release `other`: the copies agree with each other, and neither releases the
///   object the acquisition took.
/// * `guardedRelease()` — the same skeleton on one field, with a release `javac` protects: the
///   table states the protected range **and** a row over the handler's own copy (the
///   self-protection row), where the certificate proves one catch-all row and no row over the
///   handler. Measured: `[8, 30) -> 39 any` beside `[39, 41) -> 39 any`.
/// * `localLockRewritten()` — the lock lives in a local the protected body rewrites: the release
///   copies read a merged value, not the definition the acquisition read, so the one `finally`
///   would not be the object the acquisition took.
/// * `throwingRelease()` — the release is an interface call on a field whose declared type throws,
///   and it is a different field from the acquisition's: the same refusal as `lockMismatch`, kept
///   because its range begins where the table states it rather than at the acquisition.
public class LockGuardNegatives {
    private final java.util.concurrent.locks.ReentrantLock lock =
            new java.util.concurrent.locks.ReentrantLock();
    private final java.util.concurrent.locks.ReentrantLock other =
            new java.util.concurrent.locks.ReentrantLock();
    private final java.io.Closeable resource = new java.io.StringWriter();
    private final java.io.BufferedReader reader = null;
    private int count = 0;

    void lockMismatch() {
        this.lock.lock();
        try {
            this.count++;
        } finally {
            this.other.unlock();
        }
    }

    int guardedRelease() throws java.io.IOException {
        this.reader.mark(0);
        try {
            int n = 0;
            String line;
            while ((line = this.reader.readLine()) != null) {
                n++;
            }
            return n;
        } finally {
            this.reader.close();
        }
    }

    void localLockRewritten() {
        java.util.concurrent.locks.ReentrantLock held = this.lock;
        held.lock();
        try {
            held = this.other;
            this.count++;
        } finally {
            held.unlock();
        }
    }

    void throwingRelease() throws java.io.IOException {
        this.lock.lock();
        try {
            this.count++;
        } finally {
            this.resource.close();
        }
    }
}

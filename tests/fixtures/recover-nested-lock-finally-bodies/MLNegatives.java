/// The shapes beside the two admissions: each one breaks exactly one link of the certificate's own
/// proof, and each is a **verifier-valid** Java 8 class compiled from this source, so a refusal is
/// evidence about the proof rather than about hand-damaged bytes.
///
/// * `releaseOrderNotReversed()` — two acquisitions `a`, `b` and a `finally` that releases them in
///   the **acquisition** order: every release's receiver is one of the acquisitions' objects, but
///   the sequence is not the reverse the nested-lock invariant states.
/// * `unlockWithoutLock()` — the second release's receiver is a third lock the statement never
///   acquired: the copy is the release grammar, and no acquisition took its object.
/// * `throwingCallInsideRange()` — the acquisition itself lies **inside** the protected range
///   (the row covers the `lockInterruptibly` call): a throw from the call would enter the handler,
///   which would release a lock never taken, so the row set does not state the ordering this
///   admission's safety rests on.
public class MLNegatives {
    private final java.util.concurrent.locks.ReentrantLock a =
            new java.util.concurrent.locks.ReentrantLock();
    private final java.util.concurrent.locks.ReentrantLock b =
            new java.util.concurrent.locks.ReentrantLock();
    private final java.util.concurrent.locks.ReentrantLock c =
            new java.util.concurrent.locks.ReentrantLock();
    private int count = 0;

    void releaseOrderNotReversed() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
        } finally {
            this.a.unlock();
            this.b.unlock();
        }
    }

    void unlockWithoutLock() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
        } finally {
            this.b.unlock();
            this.c.unlock();
        }
    }

    void throwingCallInsideRange() throws InterruptedException {
        try {
            this.a.lockInterruptibly();
            this.count++;
        } finally {
            this.a.unlock();
        }
    }
}

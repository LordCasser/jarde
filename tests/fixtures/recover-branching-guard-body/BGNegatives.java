/// The negatives of the branching guard body: each member is the same two-lock guard over a
/// branching body, with one link of the completion's proof broken, and each is **verifier-valid**
/// (compiled from this source, so a refusal is evidence about the proof rather than about damaged
/// bytes).
///
/// The fused block's tail is the link they break:
///
/// * `tailThrow` — the tail is a `throw new …` (an allocation and an `athrow`), not the method's
///   own value-less `return`: the tail is not a run of statements this reading states, so the
///   method stays refused;
/// * `tailStoredThrow` — the tail is `throw this.stored` (`aload_0; getfield; athrow`): every
///   instruction of it is a straight one, but it ends in the method's **throw**, not in its
///   value-less return, which is the completion the certificate proves. The separate-block layout
///   refuses the same completion (`Scratch`'s `plainTailStoredThrow`, measured), so the fused
///   layout must too — the layout may not change which completions are admitted;
/// * `bodyReturn` — the branch's arm **returns from the method** inside the protected range, which
///   the certificate's own range rule refuses (a return inside the range would skip the release the
///   source's `finally` runs);
/// * `switchBody` — the body holds a **multi-way branch** (`switch`), which the region walk cannot
///   state as a statement. The method stays refused whole; its diagnostic moves from the
///   four-piece cascade the fused layout produced to the guard's own body refusal, because the
///   certificate now claims the statement before the body walk refuses it (recorded in the
///   fixture's `README.md`).
public class BGNegatives {
    private final java.util.concurrent.locks.ReentrantLock a =
            new java.util.concurrent.locks.ReentrantLock();
    private final java.util.concurrent.locks.ReentrantLock b =
            new java.util.concurrent.locks.ReentrantLock();
    private int count = 0;
    private boolean flag = false;
    private IllegalStateException stored = new IllegalStateException("stored");

    void tailThrow() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (this.flag) {
                throw new IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        throw new IllegalStateException("after");
    }

    void tailStoredThrow() {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (this.flag) {
                throw new IllegalStateException("body failed");
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
        throw this.stored;
    }

    void bodyReturn(boolean fail) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (fail) {
                return;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void switchBody(int which) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            switch (which) {
                case 0:
                    this.count++;
                    break;
                default:
                    this.count--;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }
}

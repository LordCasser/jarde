/// The measured line's far side: the protected-body shapes the walk structures **beyond** the
/// filing's own MVP note.
///
/// The filing (`openspec/changes/recover-branching-guard-body/proposal.md`) states the MVP as
/// "single branch level" and registers multi-branch bodies; the measurement says the admission is
/// wider, and this class pins what was measured rather than what was assumed:
///
/// * `twoIfs` — two branches in sequence, and `nestedIf` — a branch inside a branch: both present
///   whole. The region walk already structures them (`Sequence`/`If`, the same shapes the loop
///   family's bodies use), so the fused-tail admission presents them exactly as the source wrote
///   them; refusing them would need a branch-count rule no requirement states;
/// * `bodyLoop` — a loop in the protected body: the same admission presents it, which is the lock
///   guard family's own body shape (`LK`'s `put`/`take` present their loops where the release copy
///   keeps the range's exception edge, so the return stays a block of its own).
///
/// The far side of *these* is the multi-way branch, which the walk cannot state: it stays refused
/// (`BGNegatives.switchBody`).
public class BGProbe {
    private final java.util.concurrent.locks.ReentrantLock a =
            new java.util.concurrent.locks.ReentrantLock();
    private final java.util.concurrent.locks.ReentrantLock b =
            new java.util.concurrent.locks.ReentrantLock();
    private int count = 0;
    private boolean flag = false;

    void twoIfs(boolean first, boolean second) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (first) {
                this.count++;
            }
            if (second) {
                this.count--;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void nestedIf(boolean first, boolean second) {
        this.a.lock();
        this.b.lock();
        try {
            this.count++;
            if (first) {
                if (second) {
                    this.count++;
                }
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }

    void bodyLoop() {
        this.a.lock();
        this.b.lock();
        try {
            while (this.flag) {
                this.count++;
            }
        } finally {
            this.b.unlock();
            this.a.unlock();
        }
    }
}

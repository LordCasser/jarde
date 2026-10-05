package defpackage;

/* JADX INFO: loaded from: LK.class */
public class LK {
    private final java.util.concurrent.locks.ReentrantLock lock = new java.util.concurrent.locks.ReentrantLock();
    private final java.util.concurrent.locks.Condition notFull = this.lock.newCondition();
    private int count = 0;

    void put() throws java.lang.InterruptedException {
        this.lock.lock();
        while (this.count >= 2) {
            try {
                this.notFull.await();
            } catch (java.lang.Throwable th) {
                this.lock.unlock();
                throw th;
            }
        }
        this.count++;
        this.notFull.signalAll();
        this.lock.unlock();
    }

    int take() throws java.lang.InterruptedException {
        this.lock.lock();
        while (this.count <= 0) {
            try {
                this.notFull.await();
            } catch (java.lang.Throwable th) {
                this.lock.unlock();
                throw th;
            }
        }
        this.count--;
        this.notFull.signalAll();
        int i = this.count;
        this.lock.unlock();
        return i;
    }

    boolean tryLockQuick() {
        if (this.lock.tryLock()) {
            try {
                this.count += 10;
                return true;
            } finally {
                this.lock.unlock();
            }
        }
        return false;
    }

    public static void main(java.lang.String[] strArr) throws java.lang.Exception {
        defpackage.LK lk = new defpackage.LK();
        lk.put();
        lk.put();
        java.lang.System.out.println("" + lk.take() + "/" + lk.take() + "/" + lk.tryLockQuick());
    }
}

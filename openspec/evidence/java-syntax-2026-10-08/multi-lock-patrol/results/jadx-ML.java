package defpackage;

import java.util.concurrent.locks.Condition;
import java.util.concurrent.locks.ReentrantLock;

/* JADX INFO: loaded from: ml.jar:ML.class */
public class ML {
    private final ReentrantLock a = new ReentrantLock();
    private final ReentrantLock b = new ReentrantLock();
    private int count = 0;

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

    void interruptibly() throws InterruptedException {
        this.a.lockInterruptibly();
        try {
            this.count++;
        } finally {
            this.a.unlock();
        }
    }

    int multiAwait() throws InterruptedException {
        Condition conditionNewCondition = this.a.newCondition();
        this.a.lock();
        while (this.count <= 0) {
            try {
                conditionNewCondition.await();
            } catch (Throwable th) {
                this.a.unlock();
                throw th;
            }
        }
        if (this.count > 100) {
            conditionNewCondition.await();
        }
        this.count--;
        int i = this.count;
        this.a.unlock();
        return i;
    }

    public static void main(String[] strArr) throws Exception {
        ML ml = new ML();
        ml.nestedLocks();
        ml.interruptibly();
        System.out.println("" + ml.count);
    }
}

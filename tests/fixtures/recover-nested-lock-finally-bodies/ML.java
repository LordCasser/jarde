import java.util.concurrent.locks.*;
public class ML {
    private final ReentrantLock a = new ReentrantLock();
    private final ReentrantLock b = new ReentrantLock();
    private int count = 0;

    void nestedLocks() {                                  // 嵌套双锁：两行保护区
        a.lock();
        b.lock();
        try {
            count++;
        } finally {
            b.unlock();
            a.unlock();
        }
    }

    void interruptibly() throws InterruptedException {    // lockInterruptibly 形
        a.lockInterruptibly();
        try {
            count++;
        } finally {
            a.unlock();
        }
    }

    int multiAwait() throws InterruptedException {        // 同锁多等待点（两处 await）
        Condition notZero = a.newCondition();
        a.lock();
        try {
            while (count <= 0) { notZero.await(); }
            if (count > 100) { notZero.await(); }
            count--;
            return count;
        } finally {
            a.unlock();
        }
    }

    public static void main(String[] args) throws Exception {
        ML m = new ML();
        m.nestedLocks();
        m.interruptibly();
        System.out.println("" + m.count);
    }
}

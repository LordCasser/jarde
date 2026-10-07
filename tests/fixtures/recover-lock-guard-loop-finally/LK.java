import java.util.concurrent.locks.*;
public class LK {
    private final ReentrantLock lock = new ReentrantLock();
    private final Condition notFull = lock.newCondition();
    private int count = 0;
    void put() throws InterruptedException {                       // 经典 lock/try/finally-unlock + await 条件队列
        lock.lock();
        try {
            while(count >= 2){ notFull.await(); }
            count++;
            notFull.signalAll();
        } finally {
            lock.unlock();
        }
    }
    int take() throws InterruptedException {
        lock.lock();
        try {
            while(count <= 0){ notFull.await(); }
            count--;
            notFull.signalAll();
            return count;
        } finally {
            lock.unlock();
        }
    }
    boolean tryLockQuick(){
        if(lock.tryLock()){                                        // tryLock 守卫形
            try { count = count + 10; return true; }
            finally { lock.unlock(); }
        }
        return false;
    }
    public static void main(String[] a) throws Exception {
        LK b = new LK();
        b.put(); b.put();
        System.out.println(""+b.take()+"/"+b.take()+"/"+b.tryLockQuick());
    }
}

public class WN {
    private final Object lock = new Object();
    private boolean ready = false; private int value = 0;
    synchronized void set(int v){ value = v; ready = true; notifyAll(); }     // 同步方法 + notifyAll
    synchronized int await() throws InterruptedException { while(!ready){ wait(); } return value; }  // wait 循环（经典条件等待）
    int blockForm(int v){ synchronized(lock){ value = v; ready = true; lock.notifyAll(); return value; } }  // 同步块形
    int mixed(){ int a; synchronized(this){ a = value; } return a + 1; }     // 同步块读
    public static void main(String[] a) throws Exception {
        WN w = new WN();
        w.set(42);
        System.out.println(""+w.await()+"/"+w.blockForm(7)+"/"+w.mixed());
    }
}

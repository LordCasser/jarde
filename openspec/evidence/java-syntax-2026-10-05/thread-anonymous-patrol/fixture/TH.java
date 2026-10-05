public class TH {
    volatile boolean stop = false;                                  // volatile 标志（跨线程可见）
    int ticks = 0;
    Thread makeWorker(){                                            // 匿名 Thread 子类
        return new Thread(){
            @Override public void run(){
                while(!stop){ ticks++; if(ticks > 100){ break; } }
            }
        };
    }
    Runnable makeTask(){                                            // 匿名 Runnable（lambda 前时代）
        return new Runnable(){
            int local = 0;
            @Override public void run(){ local++; System.out.println("task:" + local); }
        };
    }
    static void joinAll(java.util.List<Thread> ts) throws InterruptedException {
        for(Thread t : ts){ t.join(); }
    }
    public static void main(String[] a) throws Exception {
        TH h = new TH();
        Thread w = h.makeWorker(); w.start(); w.join();
        Runnable r = h.makeTask(); r.run(); r.run();
        System.out.println(""+h.ticks);
        joinAll(java.util.Arrays.asList(w));
    }
}

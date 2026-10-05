public class TL {
    static final ThreadLocal<String> CTX = new ThreadLocal<String>(){          // 匿名子类 initialValue 覆写
        protected String initialValue(){ return "none"; }
    };
    static final ThreadLocal<Integer> SEQ = ThreadLocal.withInitial(() -> 0);   // withInitial lambda 工厂
    static final InheritableThreadLocal<String> PARENT = new InheritableThreadLocal<String>();  // 继承形
    static String withCtx(String v, Runnable body){                             // set→body→remove 保证清理
        String old = CTX.get();
        CTX.set(v);
        try { body.run(); } finally { CTX.set(old); }
        return CTX.get();
    }
    static int bump(){ int v = SEQ.get() + 1; SEQ.set(v); return v; }
    public static void main(String[] a) throws Exception {
        PARENT.set("root");
        final String[] got = new String[1];
        Thread t = new Thread(){ public void run(){ got[0] = PARENT.get(); } };  // 匿名 Thread 捕获
        t.start(); t.join();
        System.out.println(""+CTX.get()+"/"+withCtx("job", new Runnable(){ public void run(){ } })+"/"+bump()+bump()+bump()+"/"+got[0]);
    }
}

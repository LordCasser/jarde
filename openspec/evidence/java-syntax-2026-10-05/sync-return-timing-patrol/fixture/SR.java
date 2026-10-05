public class SR {
    static final Object LOCK = new Object();
    static int retInside(int v){                                  // return 在 monitor 内
        synchronized(LOCK){ return v * 2; }
    }
    static int localAcross(int v){                                // 局部在 monitor 内写、锁外读
        int r = 0;
        synchronized(LOCK){ r = v + 1; }
        return r;
    }
    static void voidBody(int v){                                  // void 体（对照）
        synchronized(LOCK){ if(v < 0){ throw new IllegalArgumentException("neg"); } }
    }
    static String nestedLock(int v){                              // 嵌套 monitor + 双锁
        synchronized(LOCK){
            synchronized(SR.class){ return "n" + v; }
        }
    }
    public static void main(String[] a){
        System.out.println(""+retInside(21)+"/"+localAcross(41)+"/"+nestedLock(5));
        try { voidBody(-1); } catch(IllegalArgumentException e){ System.out.println("caught:"+e.getMessage()); }
    }
}

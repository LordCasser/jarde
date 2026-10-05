public class SG {
    static class Svc {                                                // 服务本体
        private int calls = 0;
        int bump(){ return ++calls; }
    }
    private static class Holder {                                     // 惯用法 1：lazy holder（按需类初始化）
        static final Svc INSTANCE = new Svc();
    }
    static Svc holder(){ return Holder.INSTANCE; }

    private static volatile Svc dcl;                                  // 惯用法 2：DCL 双重检查锁
    static Svc dcl(){
        if(dcl == null){
            synchronized(SG.class){
                if(dcl == null){ dcl = new Svc(); }
            }
        }
        return dcl;
    }
    public static void main(String[] a){ System.out.println(""+(holder() == holder())+"/"+(dcl() == dcl())+"/"+holder().bump()+"/"+dcl().bump()); }
}

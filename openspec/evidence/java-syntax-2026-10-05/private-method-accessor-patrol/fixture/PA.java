public class PA {
    private int secret(int x){ return x * 3; }                    // 外类私有方法
    static class In {                                              // 静态嵌套类调用外类私有方法（经参数）
        static int use(PA p, int v){ return p.secret(v); }         // javac 合成 access$100? （经实例）
    }
    class InMem {                                                  // 成员内部类
        int use(int v){ return PA.this.secret(v + 1); }            // 经 outer this
    }
    private static int ssecret(int x){ return x + 7; }             // 私有静态方法
    static class InS { static int use(int v){ return ssecret(v); } }  // 静态嵌套调用私有静态
    public static void main(String[] a){ PA p = new PA(); System.out.println(""+In.use(p, 4)+"/"+p.new InMem().use(4)+"/"+InS.use(10)); }
}

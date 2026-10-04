public class IB {
    static int sc;
    int ic;
    static { sc = 41; sc = sc + 1; }                 // 静态初始化块（多语句）
    { ic = 10; }                                      // 实例初始化块
    { ic = ic + 5; }                                  // 第二个实例块（按序并入构造器）
    IB(int extra){ ic += extra; }
    IB(){ this(1); }
    static int more() { sc += 100; return sc; }       // clinit 与 static 方法并存
    public static void main(String[] a){ IB x = new IB(); System.out.println(sc+"/"+x.ic+"/"+more()); }
}

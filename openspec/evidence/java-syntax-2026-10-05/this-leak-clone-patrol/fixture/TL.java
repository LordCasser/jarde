public class TL {
    static TL inst;
    TL(){ TL.inst = this; }                     // ctor 中 this 泄漏（静态字段）
    TL(TL other){ other.inst = this; }          // 经参数泄漏
    int[] data = {1, 2, 3};
    int[] cloned(){ return data.clone(); }      // 数组 clone
    static int sum(){ TL t = new TL(); int s = 0; for(int i = 0; i < t.data.length; i++){ s += t.data[i]; } return s; }  // 长度边界循环
    static boolean selfRef(){ TL a = new TL(); return a == TL.inst; }   // 自引用比较
    public static void main(String[] a){ System.out.println(""+sum()+"/"+clonedCheck()+"/"+selfRef()); }
    static int clonedCheck(){ TL t = new TL(); int[] c = t.cloned(); c[0] = 99; return t.data[0]; }  // clone 独立性
}

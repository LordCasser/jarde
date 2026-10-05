public class TX {
    interface Sup<T> { T get(); }
    interface Act { void run(); }
    private int base = 10;
    int viaThisRef(){ Sup<Integer> s = this::compute; return s.get(); }        // this::method（实例捕获）
    int viaLambda(){ Sup<Integer> s = () -> this.base + 1; return s.get(); }   // lambda 内 this 引用
    int viaNested(){ Act a = () -> { System.out.println(this.base); }; a.run(); return this.base + 2; }  // lambda 语句体内 this
    int compute(){ return base * 2; }
    int bareThis(){ return this.base + 3; }                                     // 裸 this 字段（对照）
    public static void main(String[] a){ TX t = new TX(); System.out.println(""+t.viaThisRef()+"/"+t.viaLambda()+"/"+t.viaNested()+"/"+t.bareThis()); }
}

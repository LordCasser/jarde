public class OQ {
    private int v = 7;
    private static int sv = 8;
    int get(){ return v; }
    static int sget(){ return sv; }
    class Inner {
        int viaThis(){ return OQ.this.get(); }              // Outer.this 方法
        int viaBare(){ return get(); }                       // 裸调用（隐式 this$0）
        int viaField(){ return OQ.this.v; }                  // Outer.this 字段
        OQ back(){ return OQ.this; }                         // 返回外类实例
    }
    static class Nest {
        static int viaStatic(){ return OQ.sget(); }          // 静态嵌套访问外类静态
        int viaInst(OQ o){ return o.get(); }                 // 静态嵌套经参数访问外类实例
    }
    public static void main(String[] a){ OQ o = new OQ(); Inner i = o.new Inner(); System.out.println(""+i.viaThis()+"/"+i.viaBare()+"/"+i.viaField()+"/"+i.back().v+"/"+Nest.viaStatic()+"/"+new Nest().viaInst(o)); }
}

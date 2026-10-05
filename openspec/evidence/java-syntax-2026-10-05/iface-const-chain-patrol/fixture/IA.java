public interface IA {
    int BASE = 1;
    interface Inner { int DEEP = 10; }                        // 接口内嵌套接口（常量持有者）
}
interface IB extends IA { int MID = 2; }                       // 继承链中间
class IC implements IB {
    int viaInherit(){ return BASE + MID; }                     // 跨链继承常量（两跳）
    int viaNested(){ return IA.Inner.DEEP; }                   // 嵌套接口持有者限定
    int viaDirect(){ return IA.BASE; }                          // 限定形
    static final int LOCAL_CONST = IA.Inner.DEEP * 2;          // 常量折叠链（编译期）
    public static void main(String[] a){ IC c = new IC(); System.out.println(""+c.viaInherit()+"/"+c.viaNested()+"/"+c.viaDirect()+"/"+LOCAL_CONST); }
}

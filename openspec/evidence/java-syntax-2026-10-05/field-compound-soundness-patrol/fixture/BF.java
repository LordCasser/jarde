public class BF {
    private int flags = 0;                                    // 位标志字段
    void enable(int bit){ flags |= 1 << bit; }               // 置位（复合或+移位）
    void disable(int bit){ flags &= ~(1 << bit); }           // 清位（复合与+取反移位）
    boolean isSet(int bit){ return (flags & (1 << bit)) != 0; }   // 测试位
    boolean isEmpty(){ return flags == 0; }
    int count(){ int c = 0; int f = flags; while(f != 0){ c += f & 1; f >>>= 1; } return c; }  // popcount 扫描
    public static void main(String[] a){
        BF b = new BF();
        b.enable(0); b.enable(3); b.enable(5);
        System.out.println(""+b.isSet(0)+"/"+b.isSet(1)+"/"+b.isSet(3)+"/"+b.isEmpty()+"/"+b.count());
        b.disable(3);
        System.out.println(""+b.isSet(3)+"/"+b.count());
    }
}

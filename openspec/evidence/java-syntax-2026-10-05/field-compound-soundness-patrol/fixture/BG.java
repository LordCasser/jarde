public class BG {
    private static int sflags = 0;
    static void senable(int bit){ sflags |= 1 << bit; }        // 静态复合（无 receiver dup）
    static boolean sIs(int bit){ return (sflags & (1 << bit)) != 0; }
    int ienable2(int bit){ flags |= 1 << bit; return flags; }   // 实例复合但值被消费（返回）
    int flags = 0;
    public static void main(String[] a){ BG.senable(0); BG.senable(3); System.out.println(""+BG.sIs(0)+"/"+BG.sIs(1)); BG g = new BG(); g.ienable2(2); System.out.println(g.ienable2(4)); }
}

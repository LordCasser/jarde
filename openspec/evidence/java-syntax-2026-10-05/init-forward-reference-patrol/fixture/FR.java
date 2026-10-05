public class FR {
    static int early = initEarly();             // clinit 首项调方法（读 LATER——经方法的前向引用合法）
    static int LATER = 100;
    static int initEarly(){ return LATER + 2; } // 读到 LATER 的默认值 0（clinit 序可观察）
    int ix = iInit();
    int iy = 30;
    int iInit(){ return iy + 1; }               // 实例同形（读到 0）
    static final int C = 5;                      // 编译期常量（内联到用点）
    static int usesC(){ return C + 1; }          // C 内联为 5
    public static void main(String[] args){ FR f = new FR(); System.out.println(""+early+"/"+f.ix+"/"+usesC()+"/"+LATER); }
}

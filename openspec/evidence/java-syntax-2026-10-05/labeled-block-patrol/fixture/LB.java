public class LB {
    static int labeledBlock(int x){ int s = 0; outer: { inner: { if(x == 1) break outer; if(x == 2) break inner; s += 1; } s += 10; } s += 100; return s; }  // 非循环标签块
    static int infBreak(int x){ int i = 0; while(true){ i++; if(i > x) break; } return i; }               // 无限 while + break
    static int infReturn(int x){ int i = 0; for(;;){ i++; if(i > x) return i; } }                          // 无限 for + return
    static int doInf(int x){ int i = 0; do { i++; if(i > 100) break; } while(true); return i + x; }        // do-while(true)
    public static void main(String[] a){ System.out.println(""+labeledBlock(1)+"/"+labeledBlock(2)+"/"+labeledBlock(3)+"/"+infBreak(5)+"/"+infReturn(4)+"/"+doInf(3)); }
}

public class FR {
    // jarde 对 EX.finReturn 的呈现（逐字复制）
    static int finReturn(int arg0) {
        int local1 = arg0;
        if (arg0 == 0) {
            return -1;
        } else {
            return local1;
        }
    }
    // 原语义对照
    static int orig(int n){ try { return n; } finally { if(n==0) return -1; } }
    public static void main(String[] a){
        int bad=0;
        for(int n=-2;n<=3;n++){ if(finReturn(n)!=orig(n)){ bad++; System.out.println("DIFF at "+n+": "+finReturn(n)+" vs "+orig(n)); } }
        System.out.println(bad==0 ? "finReturn 呈现与原语义等价（6 值全一致）✓" : ("MISMATCH x"+bad));
    }
}

public class BW {
    static boolean xor(boolean a, boolean b){ return a ^ b; }                 // boolean xor
    static int ixor(int a, int b){ return a ^ b; }                             // int xor
    static boolean mix(boolean[] f){ boolean r = false; for(boolean x : f) r ^= x; return r; } // boolean xor 累积
    static int shConst(int x){ return x << 33; }                               // 移位量超宽（int 移位 &31）
    static int shFold(){ return 1 << 31; }                                     // 常量移位折叠
    static boolean andNot(boolean a, boolean b){ return a & !b; }              // andNot 布尔
    static int bits(int x){ int n=0; while(x!=0){ x &= x-1; n++; } return n; } // Kernighan 位计数（循环位技巧）
    public static void main(String[] a){ System.out.println(""+xor(true,false)+"/"+ixor(6,3)+"/"+mix(new boolean[]{true,true,true})+"/"+shConst(1)+"/"+shFold()+"/"+andNot(true,true)+"/"+bits(0b1011)); }
}

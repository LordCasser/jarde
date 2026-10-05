public class XS {
    static int[] xorSwap(int[] p){                        // XOR 交换（经典无 temp）
        p[0] ^= p[1];
        p[1] ^= p[0];
        p[0] ^= p[1];
        return p;
    }
    static int min3(int a, int b, int c){ return a < b ? (a < c ? a : c) : (b < c ? b : c); }   // 嵌套三元 min
    static int max3(int a, int b, int c){ return a > b ? (a > c ? a : c) : (b > c ? b : c); }
    static int clamp(int v, int lo, int hi){ return v < lo ? lo : (v > hi ? hi : v); }          // clamp
    static int abs(int v){ if(v < 0){ return -v; } return v; }                                  // 分支 abs
    static int gcd(int a, int b){ while(b != 0){ int t = a % b; a = b; b = t; } return a; }     // Euclid 循环内 temp
    public static void main(String[] a){
        System.out.println(""+xorSwap(new int[]{3,8})[0]+"/"+min3(4,2,9)+"/"+max3(4,2,9)+"/"+clamp(15,0,10)+"/"+abs(-7)+"/"+gcd(48,18));
    }
}

public class FA {
    int total = 0; int count = 0;
    void add(int x){ total += x; count++; }                      // 单调用形（已知 critical 面）
    void addAll(int[] xs){ for(int x : xs){ total += x; count++; } }   // 循环累积（最常见真实形）
    static int statTotal = 0;
    static void statAddAll(int[] xs){ for(int x : xs){ statTotal += x; } }   // 静态循环累积（恢复面）
    int localAcc(int[] xs){ int t = 0; for(int x : xs){ t += x; } return t; }   // 局部循环累积（恢复面）
    public static void main(String[] a){ FA f = new FA(); f.addAll(new int[]{1,2,3}); statAddAll(new int[]{10,20}); System.out.println(""+f.total+"/"+f.count+"/"+statTotal+"/"+new FA().localAcc(new int[]{5,6})); }
}

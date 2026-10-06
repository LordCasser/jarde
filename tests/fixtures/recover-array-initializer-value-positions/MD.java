public class MD {
    static int[][] jagged = { {1}, {2, 3}, {4, 5, 6} };                 // 锯齿初始化器（静态字段）
    static int sumJag(){ int s = 0; for(int[] r : jagged) for(int v : r) s += v; return s; }
    static int[][] reg = new int[2][3];                                  // 规则创建
    static int regSet(){ reg[1][2] = 9; return reg[1][2]; }
    static int[][] partial = new int[2][];                               // 部分创建（外层定、内层 deferred）
    static int partSet(){ partial[0] = new int[]{7}; partial[1] = new int[2]; return partial[0][0] + partial[1].length; }
    static int[][] mkJagged(){ return new int[][]{ {8}, {9, 10} }; }     // 方法内锯齿字面
    public static void main(String[] a){ System.out.println(""+sumJag()+"/"+regSet()+"/"+partSet()+"/"+mkJagged()[1][1]); }
}

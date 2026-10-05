public class NL {
    static String matrix(int[][][] m){                                  // 三层矩阵扫描+标签出口组合
        StringBuilder sb = new StringBuilder();
        outer:
        for(int i = 0; i < m.length; i++){
            for(int j = 0; j < m[i].length; j++){
                for(int k = 0; k < m[i][j].length; k++){
                    int v = m[i][j][k];
                    if(v < 0){ continue outer; }                        // 内层直跳外层
                    if(v == 99){ break outer; }                         // 内层终止全部
                    if(v == 50){ continue; }                            // 内层自身
                    sb.append(v).append(",");
                }
                sb.append("|");
            }
            sb.append(";");
        }
        return sb.toString();
    }
    static int findMid(int[][] m, int t){                               // 中层标签
        int hits = 0;
        mid:
        for(int i = 0; i < m.length; i++){
            for(int j = 0; j < m[i].length; j++){
                if(m[i][j] == t){ hits++; continue mid; }
                if(m[i][j] < 0){ break mid; }
            }
        }
        return hits;
    }
    public static void main(String[] a){
        int[][][] m = {{{1,2},{3,-1,4}},{{5,50,6},{7,99,8}},{{9}}};
        System.out.println(matrix(m)+"/"+findMid(new int[][]{{1,2},{3,2,5},{-1,9}},2));
    }
}

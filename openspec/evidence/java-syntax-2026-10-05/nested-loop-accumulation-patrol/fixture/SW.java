public class SW {
    static int a = 1, b = 2;
    static void swapFields(){ int t = a; a = b; b = t; }              // 静态字段 swap（temp 局部）
    static int[] swapArr(int[] xs){ int t = xs[0]; xs[0] = xs[1]; xs[1] = t; return xs; }   // 数组元素 swap
    static int[] manualCopy(int[] src){ int[] dst = new int[src.length]; for(int i = 0; i < src.length; i++){ dst[i] = src[i]; } return dst; }   // 手动数组复制
    static int sum2d(int[][] m){ int s = 0; for(int i = 0; i < m.length; i++){ for(int j = 0; j < m[i].length; j++){ s += m[i][j]; } } return s; }   // 二维遍历
    public static void main(String[] x){ swapFields(); System.out.println(""+a+","+b+"/"+java.util.Arrays.toString(swapArr(new int[]{7,9}))+"/"+java.util.Arrays.toString(manualCopy(new int[]{4,5}))+"/"+sum2d(new int[][]{{1,2},{3,4}})); }
}

import java.util.*;
public class PL3 {
    static int[] pqLambda(int[] arg0){
        java.util.PriorityQueue local1;                       // 渲染真实形：声明无赋值
        int[] local2; int local3;
        local2 = arg0;
        for (int local5 : local2) { local1.offer(Integer.valueOf(local5)); }
        local2 = new int[arg0.length];
        local3 = 0;
        while (local3 < local2.length) { local2[local3] = ((Integer) local1.poll()).intValue(); local3 = local3 + 1; }
        return local2;
    }
    public static void main(String[] a){ System.out.println(java.util.Arrays.toString(pqLambda(new int[]{3,1,2}))); }
}

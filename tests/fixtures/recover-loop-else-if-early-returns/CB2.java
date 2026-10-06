public class CB2 {
    static int exitVal(int[] xs, int k){                        // 与 CB.loopElseIfRet 同形但出口消费 lo
        int lo = 0, hi = xs.length - 1;
        while(lo <= hi){ int m = (lo + hi) >>> 1; int v = xs[m];
            if(v < k){ lo = m + 1; } else if(v > k){ hi = m - 1; } else { return m; } }
        return -(lo + 1);                                       // 出口消费循环变量（bsearch 唯一差异）
    }
    static int exitVal2(int[] xs){                              // 无 else-if 链的出口消费
        int i = 0;
        while(i < xs.length){ if(xs[i] < 0){ return i; } i = i + 1; }
        return -(i + 1);
    }
    public static void main(String[] a){ System.out.println(""+exitVal(new int[]{1,3,5}, 4)+"/"+exitVal2(new int[]{1,-2,3})); }
}

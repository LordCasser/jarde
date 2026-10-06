public class CB {
    static int loopElseIfRet(int[] xs, int k){                 // while + else-if + 早退（bsearch 形）
        int lo = 0, hi = xs.length - 1;
        while(lo <= hi){ int m = (lo + hi) >>> 1; int v = xs[m];
            if(v < k){ lo = m + 1; } else if(v > k){ hi = m - 1; } else { return m; } }
        return -1;
    }
    static int loopElseIfNoRet(int[] xs){                      // while + else-if 无早退（对照）
        int lo = 0, r = 0;
        while(lo < xs.length){ int v = xs[lo];
            if(v < 0){ lo = lo + 1; } else if(v > 0){ r = r + v; lo = lo + 1; } else { lo = lo + 1; } }
        return r;
    }
    static int loopIfElseRet(int[] xs, int k){                 // while + 单 if/else + 早退（无 else-if）
        int i = 0;
        while(i < xs.length){ if(xs[i] == k){ return i; } else { i = i + 2; } }
        return -1;
    }
    static int noLoopElseIfRet(int x){                          // 无循环 else-if + 早退
        if(x < 0){ return -1; } else if(x > 100){ return 100; } else { return x; }
    }
    public static void main(String[] a){ System.out.println(""+loopElseIfRet(new int[]{1,3,5}, 3)+"/"+loopElseIfNoRet(new int[]{-1,0,4})+"/"+loopIfElseRet(new int[]{2,4,6}, 4)+"/"+noLoopElseIfRet(50)); }
}

public class BS {
    static int bsearch(int[] xs, int key){                              // 二分查找（闭区间收缩）
        int lo = 0, hi = xs.length - 1;
        while(lo <= hi){
            int mid = (lo + hi) >>> 1;
            int v = xs[mid];
            if(v < key){ lo = mid + 1; }
            else if(v > key){ hi = mid - 1; }
            else { return mid; }
        }
        return -(lo + 1);                                               // 插入点编码
    }
    static int[] twoPtr(int[] xs){                                      // 双指针原地去零
        int w = 0;
        for(int r = 0; r < xs.length; r++){ if(xs[r] != 0){ xs[w++] = xs[r]; } }
        return java.util.Arrays.copyOf(xs, w);
    }
    static java.util.Map<Integer,Integer> memo = new java.util.HashMap<>();
    static int fib(int n){                                              // 递归+记忆化
        if(n < 2){ return n; }
        Integer hit = memo.get(n);
        if(hit != null){ return hit; }
        int r = fib(n - 1) + fib(n - 2);
        memo.put(n, r);
        return r;
    }
    public static void main(String[] a){ System.out.println(""+bsearch(new int[]{1,3,5,7}, 5)+"/"+bsearch(new int[]{1,3}, 4)+"/"+java.util.Arrays.toString(twoPtr(new int[]{0,1,0,2,3,0}))+"/"+fib(40)); }
}

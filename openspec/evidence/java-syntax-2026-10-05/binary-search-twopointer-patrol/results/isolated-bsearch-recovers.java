public class BX {                                 // bsearch 原方法逐字（无 memo 字段）
    static int bsearch(int[] xs, int key){
        int lo = 0, hi = xs.length - 1;
        while(lo <= hi){
            int mid = (lo + hi) >>> 1;
            int v = xs[mid];
            if(v < key){ lo = mid + 1; }
            else if(v > key){ hi = mid - 1; }
            else { return mid; }
        }
        return -(lo + 1);
    }
    public static void main(String[] a){ System.out.println(bsearch(new int[]{1,3,5,7}, 5)); }
}

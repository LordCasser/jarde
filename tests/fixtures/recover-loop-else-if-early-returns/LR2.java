public class LR2 {
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
    static int[] sample(){
        int[] xs = new int[4];
        xs[0] = 1; xs[1] = 3; xs[2] = 5; xs[3] = 7;
        return xs;
    }
    public static void main(String[] a){
        int[] xs = sample();
        System.out.println("" + bsearch(xs, 5) + "/" + bsearch(xs, 4));
    }
}

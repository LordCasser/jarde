public class DS {
    static boolean condAssign(int x){ return (x = x + 1) > 0; }
    static int deadLocal(int x){
        int y;
        if ((y = x + 1) > 0) { return 1; }
        return 0;
    }
    static int split(int x){
        if ((x = x + 1) > 0) { return x; }
        return -1;
    }
    static int splitLocal(int x){
        int y;
        if ((y = x + 1) > 0) { return y; }
        return -1;
    }
    static boolean deadChain(int x){
        int y;
        boolean b = (y = x + 1) > 0 && x > 10;
        return b;
    }
    public static void main(String[] a){
        System.out.println(""+condAssign(0)+"/"+deadLocal(0)+"/"+split(0)+"/"+splitLocal(0)+"/"+deadChain(5));
    }
}

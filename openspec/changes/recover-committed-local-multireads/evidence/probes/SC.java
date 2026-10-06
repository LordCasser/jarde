public class SC {
    static int[] data(){ return new int[]{1,0,3}; }
    static boolean hasZero(){ for (int x : data()) { if (x == 0) { return false; } } return true; }
    public static void main(String[] a){ System.out.println(hasZero()); }
}

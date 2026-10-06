public class CMP {
    public static void main(String[] a){
        Object p = new Object();
        Object q = new Object();
        System.out.println("eq=" + (p == q) + " tag=" + p.hashCode());
    }
}

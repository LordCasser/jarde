public class P08_twr implements AutoCloseable {
    public void close(){}
    static String one(){ try (P08_twr r = new P08_twr()) { return "in"; } }
    public static void main(String[] a){ System.out.println(one()); }
}

public class NEG {
    static String[] src = {"a", "b", null};
    static int pos = 0;
    static String read(){ return pos < src.length ? src[pos++] : null; }
    static int liveLine(){
        int n = 0;
        String line;
        while ((line = read()) != null) { n += line.length(); }
        return n;
    }
    static boolean shortChain(int x){
        boolean b = (x = x + 1) > 0 && x > 10;
        return b;
    }
    public static void main(String[] a){ System.out.println(""+liveLine()); }
}

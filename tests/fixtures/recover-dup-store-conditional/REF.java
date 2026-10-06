public class REF {
    static String[] src = {"a", "b", null};
    static int pos = 0;
    static String read(){ return pos < src.length ? src[pos++] : null; }
    static int deadLine(){
        int n = 0;
        String line;
        while ((line = read()) != null) { n++; }
        return n;
    }
    public static void main(String[] a){ System.out.println(""+deadLine()); }
}

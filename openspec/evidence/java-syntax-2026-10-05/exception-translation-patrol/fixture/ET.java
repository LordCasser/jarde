public class ET {
    static int wrap(String k){
        try { return Integer.parseInt(k); }
        catch(NumberFormatException e){ throw new IllegalStateException("bad input: " + k, e); }
    }
    static String causeChain(String k){
        try { System.out.println(wrap(k)); return "ok"; }
        catch(IllegalStateException e){ return e.getCause().getClass().getSimpleName() + "/" + e.getMessage(); }
    }
    static int initThrow(String k){
        try { return valueOf(k); }
        catch(RuntimeException e){ return -1; }
    }
    static int valueOf(String k){ return k.length() == 1 ? Integer.parseInt(k) : 0; }
    public static void main(String[] a){
        System.out.println(causeChain("x"));
        causeChain("7");
        System.out.println(initThrow("z9"));
    }
}

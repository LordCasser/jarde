public class C_sync {
    private int v;
    int bump(){ synchronized(this){ v++; return v; } }
    static int stat(Object o){ synchronized(o){ return o.hashCode(); } }
    public static void main(String[] a){ C_sync c=new C_sync(); c.bump(); System.out.println(c.bump()+"/"+stat(c)); }
}

public class SG {
    String field = "f";
    int flags = 0;
    void addSimple(String s){ field += s; }
    void orSimple(int v){ flags |= v; }
    void addComplex(String s){ field += "[" + s + "]"; }
    void orComplex(int bit){ flags |= 1 << bit; }
}

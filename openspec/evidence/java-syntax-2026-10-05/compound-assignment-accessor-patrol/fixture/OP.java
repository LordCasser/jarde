public class OP {
    private int i = 10; private long l = 10L; private boolean b = true; private String s = "x";
    class N {
        void p(){ i += 1; i -= 2; i *= 3; i /= 2; i %= 4; i &= 6; i |= 1; i ^= 3; i <<= 1; i >>= 1; i >>>= 1; }
        void q(){ l += 1L; b &= true; b |= false; s += "y"; }
    }
    public static void main(String[] a){ OP o = new OP(); N n = o.new N(); n.p(); n.q(); System.out.println(o.i+"/"+o.l+"/"+o.b+"/"+o.s); }
}

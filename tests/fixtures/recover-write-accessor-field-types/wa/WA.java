public class WA {
    private boolean flag; private int count; private long big; private double d;
    private String ref; private byte by; private short sh; private char ch; private float fl;
    class S {
        void sB(boolean v){ flag=v; } void sI(int v){ count=v; } void sL(long v){ big=v; }
        void sD(double v){ d=v; } void sS(String v){ ref=v; } void sBy(byte v){ by=v; }
        void sSh(short v){ sh=v; } void sC(char v){ ch=v; } void sF(float v){ fl=v; }
    }
    public static void main(String[] a){ WA o=new WA(); S s=o.new S();
        s.sB(true); s.sI(1); s.sL(2L); s.sD(3.0); s.sS("x"); s.sBy((byte)4); s.sSh((short)5); s.sC('c'); s.sF(6.0f);
        System.out.println(""+o.flag+o.count+o.big+o.d+o.ref+o.by+o.sh+o.ch+o.fl); }
}

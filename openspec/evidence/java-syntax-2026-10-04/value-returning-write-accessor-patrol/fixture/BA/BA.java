public class BA {
    private boolean flag = false;   // boolean 字段 → 应走 BooleanAccessorAssignment
    private int count = 0;          // int 字段 → 预期被拒
    class S { void setB(boolean b){ flag = b; } void setI(int i){ count = i; } }
    public static void main(String[] a){ BA o=new BA(); o.new S().setB(true); o.new S().setI(7); System.out.println(o.flag+"/"+o.count); }
}

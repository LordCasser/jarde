public class CI2 {
    static boolean plain(boolean b){ return b ? true : false; }           // 裸 boolean 条件（对照——分支同为字面量）
    static int plainInt(boolean b){ return b ? 1 : 0; }                     // int 字面量分支（对照）
    static Boolean unboxTern(Boolean b){ return b ? Boolean.TRUE : Boolean.FALSE; }  // 拆箱条件+装箱分支
    public static void main(String[] a){ System.out.println(""+plain(true)+"/"+plainInt(false)+"/"+unboxTern(Boolean.TRUE)); }
}

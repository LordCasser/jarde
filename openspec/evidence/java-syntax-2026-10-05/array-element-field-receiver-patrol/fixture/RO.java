public class RO {
    static class Item { final String label; Item(String l){ label = l; } }
    static Item make(){ return new Item("m"); }
    static String viaCall(){ return make().label; }                 // 调用返回值接收者
    static String viaCallLocal(){ Item i = make(); return i.label; } // 调用结果入局部
    public static void main(String[] a){ System.out.println(""+viaCall()+"/"+viaCallLocal()); }
}

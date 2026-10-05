public class RJ {
    static class Item { final String label; Item(String l){ label = l; } }
    static final Item D = new Item("d");
    static String direct(Item it){ return it.label; }                        // 直接参数
    static String viaLocal(Item it){ Item local = it; return local.label; }   // 参数入局部再读
    public static void main(String[] a){ Item i = D; System.out.println(""+direct(i)+"/"+viaLocal(i)); }
}

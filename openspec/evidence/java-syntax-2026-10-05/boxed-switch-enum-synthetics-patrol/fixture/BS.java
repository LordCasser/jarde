public class BS {
    enum Color { RED, GREEN }
    static String boxed(Integer i){ switch(i){ case 1: return "one"; case 2: return "two"; default: return "many"; } }   // Integer switch（javac 拆箱）
    static int boxedChar(Character c){ switch(c){ case 'a': return 1; case 'b': return 2; default: return 0; } }          // Character switch
    static int viaValues(){ int n = 0; for(Color k : Color.values()){ n++; } return n; }                                   // values() 合成方法
    static Color viaValueOf(String s){ return Color.valueOf(s); }                                                          // valueOf 合成方法
    static String viaOrdinal(Color c){ return c.name() + ":" + c.ordinal(); }                                              // name/ordinal
    public static void main(String[] a){ System.out.println(""+boxed(1)+"/"+boxed(9)+"/"+boxedChar('b')+"/"+viaValues()+"/"+viaValueOf("GREEN")+"/"+viaOrdinal(Color.RED)); }
}

public class D_narrow {
    static byte add(byte b){ b += 1; return b; }        // 复合赋值隐含收窄 cast
    static char c(char ch){ ch += 2; return ch; }
    public static void main(String[] x){ System.out.println(add((byte)5)+"/"+(int)c('a')); }
}

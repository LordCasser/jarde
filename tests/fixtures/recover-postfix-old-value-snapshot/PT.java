public class PT {
    static String[] src = {"a", "b", null};
    static int pos = 0;
    static String read(){ return pos < src.length ? src[pos++] : null; }   // 静态字段后缀作数组下标（三元臂）
    public static void main(String[] a){ System.out.println("" + read() + "/" + read() + "/" + read() + "/" + pos); }
}

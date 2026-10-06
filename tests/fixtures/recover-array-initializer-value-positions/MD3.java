public class MD3 {
    static int bareIdx(){ return new int[2].length; }          // 裸数组立即 length
    static int bareIdx2(){ return new int[]{9}[0]; }            // 初始化数组立即下标
    static int[] bareRet(){ return new int[3]; }                // 裸数组返回（对照）
    public static void main(String[] a){ System.out.println(""+bareIdx()+"/"+bareIdx2()+"/"+bareRet().length); }
}

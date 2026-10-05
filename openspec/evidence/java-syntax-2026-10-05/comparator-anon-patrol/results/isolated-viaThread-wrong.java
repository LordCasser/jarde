public class VT {                                   // viaThread 渲染幸存形（Thread/匿名/start/join 全吞）
    static String viaThread(String arg0){
        StringBuilder local1;
        local1 = new StringBuilder();
        try {
        } catch (InterruptedException local3) {
        }
        return local1.toString();
    }
    public static void main(String[] a){ System.out.println("["+viaThread("T")+"]"); }
}

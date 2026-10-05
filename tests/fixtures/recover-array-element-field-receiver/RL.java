public class RL {
    String label;
    RL(String l){ label = l; }
    static final RL S = new RL("s");
    static final Inner I = new Inner("i");
    static String selfElem(RL[] xs){ return xs[0].label; }        // 当前类数组元素字段
    static class Inner { String tag; Inner(String t){ tag=t; } }
    static String innerElem(Inner[] xs){ return xs[0].tag; }      // 伴生类数组元素字段（对照）
    public static void main(String[] a){
        RL[] self = new RL[]{ S };
        Inner[] inner = new Inner[]{ I };
        System.out.println(""+selfElem(self)+"/"+innerElem(inner));
    }
}
